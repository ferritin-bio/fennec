//! Amplify CLI
use candle_examples::device;
use ferritin_core::AtomCollection;
use ferritin_plms::{AmplifyModels, AmplifyRunner}
use ferritin_plms::types::PseudoProbability;
use pdbtbx::{Format, ReadOptions};
use tauri;


#[tauri::command]
pub fn get_amplify_logits(pdb_text: &str, position: i64, temp: f32) -> Result<PseudoProbability, TauriError> {
    let device = device(false)?;
    let model = "120M";
    let amp_model = match model.as_str() {
        "120M" => AmplifyModels::AMP120M,
        "350M" => AmplifyModels::AMP350M,
        &_ => panic!("Only 2 options"),
    };


    let amprunner = AmplifyRunner::load_model(amp_model, device)?;
    let prot_sequence = args.protein_string.unwrap();

    // Runs the model and returns the full, manipulateable result
    let outputs = amprunner.run_forward(&prot_sequence);
    // Runs the model and returns the top hit from each logit
    let top_hit = amprunner.get_best_prediction(&prot_sequence);
    // Runs the model and returns the top probabilities
    let get_probabilities = amprunner.get_pseudo_probabilities(&prot_sequence);
    // Runs the model and returns the contactmap
    let contact_map = amprunner.get_contact_map(&prot_sequence);

    Ok(PseudoProbability)
}

#[tauri::command]
pub fn get_amplify_contact_map(pdb_text: &str, position: i64, temp: f32) -> Result<PseudoProbability, TauriError> {
    todo!()}
