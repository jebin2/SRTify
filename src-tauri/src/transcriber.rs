use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::fs::File;
use std::io::Read;
use anyhow::Result;

use crate::utils::{ load_selection, is_video_or_audio, extract_audio, create_srt, download_model };
use tauri::{AppHandle, Emitter};
use std::sync::{Arc, Mutex};
use std::path::Path;

#[tauri::command]
pub async fn start_transcription(
    app: AppHandle
) -> Result<(), String> {
    let model_result = load_selection("model".to_string());
    let media_file_result = load_selection("file".to_string());

    let mut model = match model_result {
        Ok(Some(m)) => m,
        Ok(None) => "whisper-base".to_string(),
        Err(e) => return Err(format!("Error loading model: {}", e)),
    };

    let media_file = match media_file_result {
        Ok(Some(f)) => f,
        Ok(None) => return Err("Media file not found".into()),
        Err(e) => return Err(format!("Error loading media file: {}", e)),
    };

    // Determine both the download URL and the actual filename
    let (download_url, model_filename) = match model.as_str() {
        "whisper-base" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin".to_string(),
            "ggml-base.en.bin".to_string()
        ),
        "whisper-tiny" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin".to_string(),
            "ggml-tiny.en.bin".to_string()
        ),
        "whisper-small" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin".to_string(),
            "ggml-small.en.bin".to_string()
        ),
        "whisper-medium" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin".to_string(),
            "ggml-medium.en.bin".to_string()
        ),
        "whisper-large-v1" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v1.bin".to_string(),
            "ggml-large-v1.bin".to_string()
        ),
        "whisper-large-v2" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v2.bin".to_string(),
            "ggml-large-v2.bin".to_string()
        ),
        "whisper-large-v3" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".to_string(),
            "ggml-large-v3.bin".to_string()
        ),
        "whisper-large-v3-turbo" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin".to_string(),
            "ggml-large-v3-turbo.bin".to_string()
        ),
        _ => {
            (model.clone(), model.clone())
        }
    };

    // Download if needed
    if download_url.starts_with("https://") {
        match download_model(&download_url, &model_filename).await {
            Ok(new_path) => {
                model = new_path.to_string_lossy().to_string();
                app.emit("info", format!("Downloaded model: {}", model_filename)).unwrap_or_else(|e| {
                    eprintln!("Emit error: {}", e);
                });
            },
            Err(e) => return Err(format!("Error downloading model: {}", e)),
        }
    }

    // Verify model file exists
    if !Path::new(&model).exists() {
        return Err(format!("Model file not found at path: {}", model));
    }

    // Run transcription
    transcribe_with_whisper(media_file, &model, app)
        .await
        .map_err(|e| e.to_string())
}

async fn transcribe_with_whisper(
    mut file_path: String, 
    model_name: &str,
    app: AppHandle
) -> Result<()> {
    if let Some("video") = is_video_or_audio(&file_path).as_deref() {
        app.emit("info", "Extracting audio").unwrap_or_else(|e| {
            eprintln!("Emit error: {}", e);
        });
        let app_clone = app.clone();
        file_path = match extract_audio(&file_path, app_clone) {
            Ok(path) => path,
            Err(e) => return Err(anyhow::anyhow!("Error in extract_audio")),
        };
        app.emit("info", "Extracted audio").unwrap_or_else(|e| {
            eprintln!("Emit error: {}", e);
        });
    }
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let samples: Vec<f32> = buffer.chunks_exact(2)
        .map(|chunk| {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            sample as f32 / 32768.0  // Normalize to [-1.0, 1.0]
        })
        .collect();

    let params = WhisperContextParameters::default();
    let ctx = WhisperContext::new_with_params(&model_name, params)?;
    let mut state = ctx.create_state()?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_print_realtime(false);
    params.set_print_progress(false);
    params.set_print_timestamps(false);
    params.set_print_special(false);

    app.emit("transcription_started", "TRANSCRIPTION_STARTED").unwrap_or_else(|e| {
        eprintln!("Emit error: {}", e);
    });
    
    let subtitles = Arc::new(Mutex::new(Vec::new()));
    let subtitles_clone = subtitles.clone();
    
    let app_clone = app.clone();
    app.emit("transcription_started", "TRANSCRIPTION_STARTED").unwrap_or_else(|e| {
        eprintln!("Emit error: {}", e);
    });
    
    params.set_segment_callback_safe(move |data: whisper_rs::SegmentCallbackData| {
        let message = format!("test: {} start_time: {:.2} end_time:{:.2}", 
            data.text.clone(), 
            data.start_timestamp as f64 * 0.01, 
            data.end_timestamp as f64 * 0.01
        );

        if let Err(e) = app_clone.emit("transcription_progress", message.clone()) {
            eprintln!("Emit error: {}", e);
        }
        println!("{}", message);
        
        if let Ok(mut subtitles) = subtitles_clone.lock() {
            subtitles.push((
                data.text.clone(), 
                data.start_timestamp as f64 * 0.01, 
                data.end_timestamp as f64 * 0.01
            ));
        }
    });
    
    state.full(params, &samples)?;
    app.emit("transcription_complete", "TRANSCRIPTION_COMPLETE").unwrap_or_else(|e| {
        eprintln!("Emit error: {}", e);
    });

    if let Err(e) = create_srt(subtitles.lock().unwrap().clone()) {
        eprintln!("Error creating SRT: {}", e);
    }
    
    Ok(())
}

#[tauri::command]
pub async fn stop_transcription() -> Result<(), String> {
    println!("Stop transcription invoked");
    Ok(())
}