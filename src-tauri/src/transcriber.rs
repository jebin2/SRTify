use anyhow::Result;
use std::fs::File;
use std::io::Read;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use std::env;
use std::fs;

use crate::utils::{
    create_srt, create_json, download_model, extract_audio, get_audio_duration, is_video_or_audio,
    load_selection,
};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn start_transcription(app: AppHandle) -> Result<(), String> {
    app.emit("transcription_started", "TRANSCRIPTION_STARTED").ok();

    let model = validate_and_load_selection(&app, "model", "Model file not found")?;
    let media_file = validate_and_load_selection(&app, "file", "Media File not found")?;
    let media_folder = validate_and_load_selection(&app, "folder", "Output Folder not found")?;

    validate_path_exists(&app, &media_file, "Media File not found at path")?;
    validate_path_exists(&app, &media_folder, "Output Folder not found at path")?;

    let (download_url, mut new_path_parent) = match_model(&model)?;

    if download_url.starts_with("https://") {
        let app_clone = app.clone();
        match download_model(&download_url, &new_path_parent, app_clone).await {
            Ok(new_path) => {
                new_path_parent = new_path.to_string_lossy().to_string();
                app.emit("success", format!("Downloaded model: {}", model))
                    .unwrap_or_else(|e| eprintln!("Emit error: {}", e));
            }
            Err(e) => {
                app.emit("error", format!("Error downloading model: {}", e))
                    .unwrap_or_else(|e| eprintln!("Emit error: {}", e));
                return Err(format!("Error downloading model: {}", e));
            }
        }
    }

    validate_path_exists(&app, &new_path_parent, "Model file not found at path")?;

    transcribe_with_whisper(media_file, &new_path_parent, app)
        .await
        .map_err(|e| e.to_string())
}

fn validate_and_load_selection(app: &AppHandle, key: &str, error_message: &str) -> Result<String, String> {
    load_selection(key.to_string())
        .map_err(|e| {
            app.emit("error", error_message).ok();
            format!("Error loading {}: {}", key, e)
        })
        .and_then(|opt| opt.ok_or_else(|| {
            app.emit("error", error_message).ok();
            error_message.to_string()
        }))
}

fn validate_path_exists(app: &AppHandle, path: &str, error_message: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        app.emit("error", format!("{}: {}", error_message, path)).ok();
        Err(format!("{}: {}", error_message, path))
    } else {
        Ok(())
    }
}

fn match_model(model: &str) -> Result<(String, String), String> {
    let temp_dir = env::temp_dir();
    let srtify_dir = temp_dir.join("srtify");
    fs::create_dir_all(&srtify_dir).expect("Failed to create srtify directory");

    let (download_url, model_filename) = match model {
        "whisper-base" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin".to_string(),
            srtify_dir.join("ggml-base.en.bin").to_string_lossy().to_string(),
        ),
        "whisper-tiny" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin".to_string(),
            srtify_dir.join("ggml-tiny.en.bin").to_string_lossy().to_string(),
        ),
        "whisper-small" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin".to_string(),
            srtify_dir.join("ggml-small.en.bin").to_string_lossy().to_string(),
        ),
        "whisper-medium" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin".to_string(),
            srtify_dir.join("ggml-medium.en.bin").to_string_lossy().to_string(),
        ),
        "whisper-large-v1" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v1.bin".to_string(),
            srtify_dir.join("ggml-large-v1.bin").to_string_lossy().to_string(),
        ),
        "whisper-large-v2" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v2.bin".to_string(),
            srtify_dir.join("ggml-large-v2.bin").to_string_lossy().to_string(),
        ),
        "whisper-large-v3" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".to_string(),
            srtify_dir.join("ggml-large-v3.bin").to_string_lossy().to_string(),
        ),
        "whisper-large-v3-turbo" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin".to_string(),
            srtify_dir.join("ggml-large-v3-turbo.bin").to_string_lossy().to_string(),
        ),
        _ => (model.to_string(), model.to_string()),
    };
    Ok((download_url, model_filename))
}

async fn transcribe_with_whisper(
    mut file_path: String,
    model_name: &str,
    app: AppHandle,
) -> Result<()> {
    if let Some(media_type) = is_video_or_audio(&file_path).as_deref() {
        if media_type == "video" || media_type == "audio" {
            app.emit("info", "Extracting/Converting audio to WAV using ffmpeg").unwrap_or_else(|e| {
                eprintln!("Emit error: {}", e);
            });
            let app_clone = app.clone();
            file_path = match extract_audio(&file_path, app_clone) {
                Ok(path) => path,
                Err(_) => return Err(anyhow::anyhow!("Error in extract_audio")),
            };
            app.emit("info", "Audio ready in WAV format").unwrap_or_else(|e| {
                eprintln!("Emit error: {}", e);
            });
        }
    }

    let mut file = File::open(file_path.clone())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let samples: Vec<f32> = buffer
        .chunks_exact(2)
        .map(|chunk| {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            sample as f32 / 32768.0 // Normalize to [-1.0, 1.0]
        })
        .collect();

    let duration = match get_audio_duration(&file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error getting audio duration: {}", e);
            return Err(e.into()); // Convert to your error type
        }
    };

    let params = WhisperContextParameters::default();
    let ctx = WhisperContext::new_with_params(&model_name, params)?;
    let mut state = ctx.create_state()?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_token_timestamps(true);
    params.set_print_realtime(false);
    params.set_print_progress(false);
    params.set_print_timestamps(false);
    params.set_print_special(false);

    let subtitles = Arc::new(Mutex::new(Vec::new()));
    let subtitles_clone = subtitles.clone();

    let app_clone = app.clone();
    app.emit("transcription_started", "TRANSCRIPTION_STARTED")
        .unwrap_or_else(|e| {
            eprintln!("Emit error: {}", e);
        });

    params.set_segment_callback_safe(move |data: whisper_rs::SegmentCallbackData| {
        let message = format!(
            "Transcription: {} start_time: {:.2} end_time:{:.2} duration: {:.2}",
            data.text.clone(),
            data.start_timestamp as f64 * 0.01,
            data.end_timestamp as f64 * 0.01,
            duration
        );

        if let Err(e) = app_clone.emit("transcription_progress", message.clone()) {
            eprintln!("Emit error: {}", e);
        }
        println!("{}", message);

        if let Ok(mut subtitles) = subtitles_clone.lock() {
            subtitles.push((
                data.text.clone(),
                data.start_timestamp as f64 * 0.01,
                data.end_timestamp as f64 * 0.01,
            ));
        }
    });

    state.full(params, &samples)?;
    app.emit("transcription_complete", "TRANSCRIPTION_COMPLETE")
        .unwrap_or_else(|e| {
            eprintln!("Emit error: {}", e);
        });

    if let Err(e) = create_srt(subtitles.lock().unwrap().clone(), app.clone()) {
        eprintln!("Error creating SRT: {}", e);
    }

    // --- Word-level extraction ---
    let mut all_segments = Vec::new();
    let n_segments = state.full_n_segments()?;

    for i in 0..n_segments {
        let segment_start = state.full_get_segment_t0(i)? as f64 * 0.01;
        let segment_end = state.full_get_segment_t1(i)? as f64 * 0.01;
        let text = state.full_get_segment_text(i)?;

        let mut words = Vec::new();
        let n_tokens = state.full_n_tokens(i)?;
        for j in 0..n_tokens {
            let token_text = state.full_get_token_text(i, j)?;
            let token_data = state.full_get_token_data(i, j)?;
            let token_t0 = token_data.t0;
            let token_t1 = token_data.t1;

            // Only include tokens that have timing info
            if token_t0 >= 0 && token_t1 >= 0 {
                words.push(serde_json::json!({
                    "word": token_text.trim(),
                    "start": token_t0 as f64 * 0.01,
                    "end": token_t1 as f64 * 0.01
                }));
            }
        }

        all_segments.push(serde_json::json!({
            "text": text.trim(),
            "start": segment_start,
            "end": segment_end,
            "words": words
        }));
    }

    // Save JSON file
    if let Err(e) = create_json(all_segments, app.clone()) {
        eprintln!("Error creating JSON: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_transcription() -> Result<(), String> {
    println!("Stop transcription invoked");
    Ok(())
}
