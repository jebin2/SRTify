use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use futures::StreamExt;
use std::env;
use std::fs;
use tauri::{AppHandle, Emitter, Manager};
use std::process::Command;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::error::Error;
use hound::WavReader;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SelectedData {
    model: Option<String>,
    file_path: Option<String>,
    folder_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaData {
    key: String,
    value: String,
}

#[tauri::command]
pub async fn select_folder() -> String {
    let result = FileDialog::new()
        .set_directory(".")
        .pick_folder();

    match result {
        Some(path) => path.display().to_string(),
        None => String::new(),
    }
}

#[tauri::command]
pub async fn select_file(is_model: bool) -> String {
    print!("{}", is_model);
    let result = if is_model {
        FileDialog::new()
            .set_directory(".")
            .add_filter("BIN files", &["bin"]) // Restrict to .bin files
            .pick_file()
    } else {
        FileDialog::new()
            .set_directory(".")
            .add_filter("Video files", &["mp4", "avi", "mkv"])  // Video files
            .add_filter("Audio files", &["mp3", "wav", "flac", "aac"])  // Audio files
            .pick_file()
    };

    match result {
        Some(path) => path.display().to_string(),
        None => String::new(),
    }
}

#[tauri::command]
pub fn save_selection(data: MetaData, app: AppHandle) -> Result<(), String> {
    let temp_dir = env::temp_dir();
    let srtify_dir = temp_dir.join("srtify");

    // Create the srtify directory if it doesn't exist
    fs::create_dir_all(&srtify_dir).expect("Failed to create srtify directory");
    let file_path = srtify_dir.join("srtify.json");

    // Load existing data or initialize with default
    let mut selected_data: SelectedData = if file_path.exists() {
        let file_content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
        let mut data: SelectedData = serde_json::from_str(&file_content).map_err(|e| e.to_string())?;

        // Ensure the model is not null, empty, or None after deserialization
        if data.model.is_none() || data.model.as_deref() == Some("null") || data.model.as_deref() == Some("") {
            data.model = Some("whisper".to_string());
        }

        data
    } else {
        SelectedData::default()
    };

    // Update the selected data based on the key
    match data.key.as_str() {
        "model" => {
            // Set the model value, ensuring it's not null, empty, or None
            selected_data.model = match data.value.as_str() {
                "" | "null" | "none" => Some("whisper".to_string()), // Default to "whisper"
                _ => Some(data.value),
            };
        }
        "file" => {
            selected_data.file_path = Some(data.value);
        }
        "folder" => {
            selected_data.folder_path = Some(data.value);
        }
        _ => {
            return Err(format!("Unknown key: {}", data.key));
        }
    }

    // Ensure the model is not None, null, or empty after all updates
    if selected_data.model.is_none() || selected_data.model.as_deref() == Some("null") || selected_data.model.as_deref() == Some("") {
        selected_data.model = Some("whisper".to_string());
    }

    // Serialize the updated data to JSON
    let state_json = serde_json::to_string(&selected_data).map_err(|e| e.to_string())?;
    let state_json_clone = state_json.clone();

    // Write the updated data to the file
    fs::write(&file_path, state_json).map_err(|e| e.to_string())?;

    // Emit the updated data to the frontend
    app.emit("info", state_json_clone).unwrap_or_else(|e| {
        eprintln!("Emit error: {}", e);
    });

    Ok(())
}

#[tauri::command]
pub fn load_selection(key: String) -> Result<Option<String>, String> {
    let temp_dir = env::temp_dir();
    let srtify_dir = temp_dir.join("srtify");

    fs::create_dir_all(&srtify_dir).expect("Failed to create srtify directory");
    let file_path = srtify_dir.join("srtify.json");

    let selected_data: SelectedData = if file_path.exists() {
        let file_content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&file_content).map_err(|e| e.to_string())?
    } else {
        return Ok(None);
    };

    match key.as_str() {
        "model" => Ok(selected_data.model.clone()),
        "file" => Ok(selected_data.file_path.clone()),
        "folder" => Ok(selected_data.folder_path.clone()),
        _ => Err(format!("Unknown key: {}", key)),
    }
}

fn get_ffmpeg_path(app: tauri::AppHandle) -> Result<PathBuf, String> {
    let target_os = std::env::consts::OS;
    let path = match target_os {
        "linux" => Ok("bin/linux-x64/ffmpeg"),
        "macos" => Ok("bin/macos-x64/ffmpeg"),
        "windows" => Ok("bin/win32-x64/ffmpeg.exe"),
        _ => Err(format!("Unsupported target OS: {}", target_os)),
    }?;
    let resource_path = app.path()
        .resolve(path, tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve FFmpeg path: {}", e))?;

    println!("resource_path:: {:?}", resource_path);
    if !resource_path.exists() {
        return Err(format!("FFmpeg not found at {:?}", resource_path));
    }

    Ok(resource_path)
}

pub fn extract_audio(video_path: &str, app: tauri::AppHandle) -> Result<String, String> {
    let ffmpeg_path = get_ffmpeg_path(app).expect("Failed to get FFmpeg path");

    if !ffmpeg_path.exists() {
        return Err(format!("FFmpeg binary not found at {:?}", ffmpeg_path).to_string());
    }
    println!("ffmpeg_path {:?}", ffmpeg_path);
    let temp_dir = env::temp_dir();
    let srtify_dir = temp_dir.join("srtify");
    fs::create_dir_all(&srtify_dir).expect("Failed to create srtify directory");
    let audio_output = srtify_dir.join("output.wav");

    if let Err(e) = fs::remove_file(&audio_output) {
        eprintln!("Failed to remove existing audio output file: {}", e);
    }

    Command::new(ffmpeg_path)
        .args(&[
            "-y",
            "-i", video_path,
            "-vn",
            "-acodec", "pcm_s16le",
            "-ar", "16000",
            "-ac", "1",
            audio_output.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to extract audio");

    Ok(audio_output.to_str().unwrap().to_string())
}

pub fn is_video_or_audio(file_path: &str) -> Option<&'static str> {
    let path = Path::new(file_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => {
            let video_exts = ["mp4", "mkv", "avi", "mov", "flv", "wmv", "webm"];
            let audio_exts = ["mp3", "wav", "flac", "aac", "ogg", "m4a"];

            if video_exts.contains(&ext) {
                Some("video")
            } else if audio_exts.contains(&ext) {
                Some("audio")
            } else {
                None
            }
        }
        None => None,
    }
}

fn sec_to_time_format(sec: f64) -> String {
    let hours = (sec / 3600.0).floor() as u32;
    let minutes = ((sec % 3600.0) / 60.0).floor() as u32;
    let seconds = (sec % 60.0).floor() as u32;
    let milliseconds = ((sec - sec.floor()) * 1000.0).round() as u32;

    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, milliseconds)
}

pub fn create_srt(subtitles: Vec<(String, f64, f64)>, app: tauri::AppHandle) -> Result<String, Box<dyn Error>> {
    let folder_res = load_selection("folder".to_string());

    let folder = match folder_res {
        Ok(Some(m)) => m,
        Ok(None) => return Err("Media file not found".into()),
        Err(e) => return Err(format!("Error loading model: {}", e).into()),
    };
    let filename = format!("{}/output.srt", folder);

    if let Err(e) = fs::remove_file(&filename) {
        eprintln!("Failed to remove existing audio output file: {}", e);
    }

    let mut file = File::create(&filename).map_err(|e| format!("Failed to create SRT file: {}", e))?;

    for (i, (text, start_sec, end_sec)) in subtitles.iter().enumerate() {
        let start_time = sec_to_time_format(*start_sec);
        let end_time = sec_to_time_format(*end_sec);

        writeln!(file, "{}\n{} --> {}\n{}\n", i + 1, start_time, end_time, text)
            .map_err(|e| format!("Failed to write to SRT file: {}", e))?;
    }
    app.emit("subtitle_created", format!("Subtitle Created :: {}", filename)).unwrap_or_else(|e| {
        eprintln!("Emit error: {}", e);
    });
    app.emit("info", "End").unwrap_or_else(|e| {
        eprintln!("Emit error: {}", e);
    });
    Ok(filename)
}

pub async fn download_model(url: &str, model_name: &str, app: AppHandle) -> Result<PathBuf, Box<dyn Error>> {
    let temp_dir = env::temp_dir();
    let srtify_dir = temp_dir.join("srtify");
    fs::create_dir_all(&srtify_dir).expect("Failed to create srtify directory");
    let model_path = srtify_dir.join(model_name);
    
    if Path::new(&model_path).exists() {
        return Ok(model_path);
    }

    // Emit download start event
    let start_event = serde_json::json!({
        "status": "download_start",
        "model": model_name,
        "progress": 0.0
    });
    app.emit("download_progress", start_event).unwrap_or_else(|e| {
        eprintln!("Emit error: {}", e);
    });

    let response = reqwest::get(url).await?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download model from URL: {}", url).into());
    }

    // Create parent directories if needed
    if let Some(parent) = Path::new(&model_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(model_path.clone())?;
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded += chunk.len() as u64;
        file.write_all(&chunk)?;
        
        // Calculate progress percentage if we know total size
        let progress = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            // Use -1 to indicate unknown total size
            -1.0
        };

        // Emit progress update
        let progress_event = serde_json::json!({
            "status": "download_progress",
            "model": model_name,
            "progress": progress,
            "downloaded": downloaded,
            "total_size": total_size
        });
        app.emit("download_progress", progress_event).unwrap_or_else(|e| {
            eprintln!("Emit error: {}", e);
        });
    }

    // Emit completion event
    let complete_event = serde_json::json!({
        "status": "download_complete",
        "model": model_name,
        "path": model_path.to_str(),
        "progress": 100
    });
    app.emit("download_progress", complete_event).unwrap_or_else(|e| {
        eprintln!("Emit error: {}", e);
    });

    Ok(model_path)
}

pub fn get_audio_duration(file_path: &str) -> Result<f64, hound::Error> {
    let reader = WavReader::open(file_path)?;
    let spec = reader.spec();
    
    let duration = reader.duration() as f64 / spec.sample_rate as f64;
    Ok(duration)
}