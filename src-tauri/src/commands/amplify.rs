//! Amplify CLI
use anyhow::Result;
use candle_examples::device;
use ferritin_core::load_structure_from_string;
use ferritin_plms::ligandmpnn::utilities::aa3to1;
use ferritin_plms::types::{ContactMap, PseudoProbability};
use ferritin_plms::{AmplifyModels, AmplifyRunner};

#[tauri::command]
pub fn get_amplify_logits(pdb_text: &str) -> Result<Vec<PseudoProbability>, String> {
    let ac = load_structure_from_string(pdb_text, "pdb").map_err(|e| e.to_string())?;

    let sequence = ac
        .iter_residues_aminoacid()
        .map(|res| res.residue_name().to_string())
        .map(|res3| aa3to1(&res3))
        .collect::<String>();

    let device = device(false).map_err(|e| e.to_string())?;
    let model = "120M"; // this should be a param
    let amp_model = match model {
        "120M" => AmplifyModels::AMP120M,
        "350M" => AmplifyModels::AMP350M,
        &_ => panic!("Only 2 options"),
    };
    let amprunner = AmplifyRunner::load_model(amp_model, device).map_err(|e| e.to_string())?;
    let pseudo_probabilities = amprunner
        .get_pseudo_probabilities(&sequence)
        .map_err(|e| e.to_string())?;
    Ok(pseudo_probabilities)
}

#[tauri::command]
pub fn get_amplify_contact_map(pdb_text: &str) -> Result<Vec<ContactMap>, String> {
    let ac = load_structure_from_string(pdb_text, "pdb").map_err(|e| e.to_string())?;

    let sequence = ac
        .iter_residues_aminoacid()
        .map(|res| res.residue_name().to_string())
        .map(|res3| aa3to1(&res3))
        .collect::<String>();
    let device = device(false).map_err(|e| e.to_string())?;
    let model = "120M"; // this should be a param
    // println!("Device: {:?}", &device);
    let amp_model = match model {
        "120M" => AmplifyModels::AMP120M,
        "350M" => AmplifyModels::AMP350M,
        &_ => panic!("Only 2 options"),
    };
    let amprunner = AmplifyRunner::load_model(amp_model, device).map_err(|e| e.to_string())?;
    let contact_map = amprunner
        .get_contact_map(&sequence)
        .map_err(|e| e.to_string())?;
    Ok(contact_map)
}
