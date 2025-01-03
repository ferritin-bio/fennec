//! Amplify CLI
use candle_examples::device;
use ferritin_core::AtomCollection;
use ferritin_plms::{AmplifyModels, AmplifyRunner};
use ferritin_plms::ligandmpnn::utilities::aa3to1;
use ferritin_plms::types::{PseudoProbability,ContactMap};
use pdbtbx::{Format, ReadOptions};
use std::io::BufReader;
use tauri::Error as TauriError;

#[tauri::command]
pub fn get_amplify_logits(pdb_text: &str) -> Result<Vec<PseudoProbability>, String> {
    let pdb_bytes = pdb_text.as_bytes();
    let reader = BufReader::new(pdb_bytes);
    let (pdb, _error) = ReadOptions::default()
        .set_format(Format::Mmcif)
        .read_raw(reader)
        .expect("Failed to parse PDB/CIF");
    let ac = AtomCollection::from(&pdb);
    let sequence = ac
        .iter_residues_aminoacid()
        .map(|res| res.res_name)
        .map(|res3| aa3to1(&res3))
        .collect::<String>();

    let device = device(false).map_err(|e| e.to_string())?;
    let model = "120M"; // this should be a param
    let amp_model = match model{
        "120M" => AmplifyModels::AMP120M,
        "350M" => AmplifyModels::AMP350M,
        &_ => panic!("Only 2 options"),
    };
    let amprunner = AmplifyRunner::load_model(amp_model, device).map_err(|e| e.to_string())?;
    let pseudo_probabilities = amprunner.get_pseudo_probabilities(&sequence).map_err(|e| e.to_string())?;
    Ok(pseudo_probabilities)
}

#[tauri::command]
pub fn get_amplify_contact_map(pdb_text: &str) -> Result<Vec<ContactMap>, String> {
    let pdb_bytes = pdb_text.as_bytes();
    let reader = BufReader::new(pdb_bytes);
    let (pdb, _error) = ReadOptions::default()
        .set_format(Format::Mmcif)
        .read_raw(reader)
        .expect("Failed to parse PDB/CIF");
    let ac = AtomCollection::from(&pdb);
    let sequence = ac
        .iter_residues_aminoacid()
        .map(|res| res.res_name)
        .map(|res3| aa3to1(&res3))
        .collect::<String>();
    let device = device(false).map_err(|e| e.to_string())?;
    let model = "120M"; // this should be a param
    println!("Device: {:?}", &device);
    let amp_model = match model{
        "120M" => AmplifyModels::AMP120M,
        "350M" => AmplifyModels::AMP350M,
        &_ => panic!("Only 2 options"),
    };
    let amprunner = AmplifyRunner::load_model(amp_model, device).map_err(|e| e.to_string())?;
    let contact_map = amprunner.get_contact_map(&sequence).map_err(|e| e.to_string())?;
    Ok(contact_map)
}
