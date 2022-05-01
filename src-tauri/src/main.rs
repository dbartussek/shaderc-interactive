#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub mod compile_shader;

use compile_shader::compile_shader;

fn main() {
    color_eyre::install().unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![compile_shader])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
