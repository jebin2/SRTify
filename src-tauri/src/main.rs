// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod transcriber;
mod local_server;
mod utils;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Spawn the local HTTP server
            tokio::spawn(async {
                local_server::start_server().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            utils::select_file,
            utils::select_folder,
            utils::save_selection,
            utils::load_selection,
            transcriber::start_transcription,
            transcriber::stop_transcription
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}