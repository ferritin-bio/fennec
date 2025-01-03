// Core Fennec Lib.
//
// This namespace exists to define the handlers we pass to Tauri.
//
mod commands;
use commands::esm2_logits::get_esm2_logits;
use commands::ligandmpnn_logits::get_ligmpnn_logits;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_ligmpnn_logits,
            get_esm2_logits
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
