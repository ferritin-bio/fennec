//! Amplify CLI
use candle_examples::device;
use ferritin_core::AtomCollection;
use ferritin_plms::{AmplifyModels, AmplifyRunner}
use ferritin_plms::types::PseudoProbability;
use pdbtbx::{Format, ReadOptions};
use std::io::BufReader;
use tauri;

#[tauri::command]
pub fn get_amplify_logits(pdb_text: &str, position: i64, temp: f32) -> Result<PseudoProbability, TauriError> {
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


    let device = device(false)?;
    let model = "120M"; // this should be a param
    let amp_model = match model{
        "120M" => AmplifyModels::AMP120M,
        "350M" => AmplifyModels::AMP350M,
        &_ => panic!("Only 2 options"),
    };
    let amprunner = AmplifyRunner::load_model(amp_model, device)?;
    let pseudo_probabilities = amprunner.get_pseudo_probabilities(&prot_sequence);
    Ok(pseudo_probabilities)
}

#[tauri::command]
pub fn get_amplify_contact_map(pdb_text: &str, position: i64, temp: f32) -> Result<PseudoProbability, TauriError> {
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


    let device = device(false)?;
    let model = "120M"; // this should be a param
    let amp_model = match model{
        "120M" => AmplifyModels::AMP120M,
        "350M" => AmplifyModels::AMP350M,
        &_ => panic!("Only 2 options"),
    };
    let amprunner = AmplifyRunner::load_model(amp_model, device)?;

    let contact_map = amprunner.get_contact_map(&prot_sequence);
    Ok(contact_map)
    }
}
