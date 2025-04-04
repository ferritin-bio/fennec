// Core Fennec Lib.
//
// This namespace exists to define the handlers we pass to Tauri.
//
mod commands;
use commands::amplify::{get_amplify_contact_map, get_amplify_logits};
use commands::dna_utils::translate_dna;
use commands::esm2_logits::get_esm2_logits;
use commands::fastx::{convert_fastq_to_fasta_tauri, get_seqstats, get_stats};
use commands::ligandmpnn_logits::get_ligmpnn_logits;
use commands::patterns::check_restriction_sites;
use commands::rnapkin::rnapkin_fn;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_restriction_sites,
            convert_fastq_to_fasta_tauri,
            get_amplify_contact_map,
            get_amplify_logits,
            get_esm2_logits,
            get_ligmpnn_logits,
            get_seqstats,
            get_stats,
            rnapkin_fn,
            translate_dna,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
