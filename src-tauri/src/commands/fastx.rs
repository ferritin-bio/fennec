use fasta::record::Definition as FastaDefinition;
use noodles_fasta as fasta;
use noodles_fastq as fastq;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

//  Get Stats -------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct FastaData {
    recordcount: i32,
    maxlength: i32,
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_stats(filename: String) -> Result<FastaData, String> {
    let mut count = 0;
    let mut maxlength = 0;

    let mut reader = fasta::io::reader::Builder
        .build_from_path(filename)
        .map_err(|e| e.to_string())?;

    for result in reader.records() {
        count += 1;
        let record = result.map_err(|e| e.to_string())?;
        let reclength = record.sequence().len();
        if reclength > maxlength {
            maxlength = reclength;
        }
    }

    Ok(FastaData {
        recordcount: count,
        maxlength: maxlength as i32,
    })
}

//  Get Advanced Stats  -------------------------------------------------------------------------------------------------------------------

// simple statistics of FASTA/Q files
// Columns:
//   1.  file      input file, "-" for STDIN
//   2.  format    FASTA or FASTQ
//   3.  type      DNA, RNA, Protein or Unlimit
//   4.  num_seqs  number of sequences
//   5.  sum_len   number of bases or residues       , with gaps or spaces counted
//   6.  min_len   minimal sequence length           , with gaps or spaces counted
//   7.  avg_len   average sequence length           , with gaps or spaces counted
//   8.  max_len   miximal sequence length           , with gaps or spaces counted
//   9.  Q1        first quartile of sequence length , with gaps or spaces counted
//   10. Q2        median of sequence length         , with gaps or spaces counted
//   11. Q3        third quartile of sequence length , with gaps or spaces counted
//   12. sum_gap   number of gaps
//   13. N50       N50. https://en.wikipedia.org/wiki/N50,_L50,_and_related_statistics#N50
//   14. Q20(%)    percentage of bases with the quality score greater than 20
//   15. Q30(%)    percentage of bases with the quality score greater than 30
//   16. GC(%)     percentage of GC content
//
// Basic is sum_len min_len avg_len max_len
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeqKitFastaData {
    filename: String,
    format: String,
    datatype: String,
    num_seqs: i32,
    sum_len: i32,
    min_len: i32,
    avg_len: f32,
    contig_lengths: Vec<i32>,
    max_len: i32,
}

#[tauri::command]
pub fn get_seqstats(filename: String) -> Result<SeqKitFastaData, String> {
    let mut count = 0;
    let mut lengths: Vec<i32> = Vec::new();

    let mut reader = fasta::io::reader::Builder
        .build_from_path(filename.clone())
        .map_err(|e| e.to_string())?;

    for result in reader.records() {
        count += 1;
        let record = result.map_err(|e| e.to_string())?;
        lengths.push(record.sequence().len() as i32);
    }

    let total: i32 = lengths.iter().sum();
    let min_value = *lengths.iter().min().unwrap_or(&0);
    let max_value = *lengths.iter().max().unwrap_or(&0);
    let avg = if count > 0 { total as f32 / count as f32 } else { 0.0 };

    Ok(SeqKitFastaData {
        filename,
        format: "Fasta".to_string(),
        datatype: "DNA".to_string(),
        num_seqs: count,
        sum_len: total,
        avg_len: avg,
        min_len: min_value,
        max_len: max_value,
        contig_lengths: lengths,
    })
}

//  Convert Fastq to Fasta -------------------------------------------------------------------------------------------------------------------

pub fn convert_fastq_to_fasta(input_path: &str, output_path: &str) -> io::Result<()> {
    let mut reader = File::open(input_path).map(BufReader::new).map(fastq::io::Reader::new)?;
    let mut fasta_writer = fasta::io::Writer::new(BufWriter::new(File::create(output_path)?));

    for result in reader.records() {
        let record = result?;
        let description = record.description();
        let fasta_definition = FastaDefinition::new(
            record.name().to_owned(),
            (!description.is_empty()).then(|| description.to_owned()),
        );
        let fasta_record = fasta::Record::new(
            fasta_definition,
            fasta::record::Sequence::from(record.sequence().to_vec()),
        );
        fasta_writer.write_record(&fasta_record)?;
    }

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn convert_fastq_to_fasta_tauri(input_path: &str, output_path: &str) -> Result<String, String> {
    convert_fastq_to_fasta(input_path, output_path)
        .map(|_| format!("Fasta file {output_path} has been created!"))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_stats() {
        let result = get_stats("testdata/fastx/small.fasta".to_string());
        let FastaData {
            maxlength,
            recordcount,
        } = result.expect("Failed to get stats");
        assert_eq!(maxlength, 34);
        assert_eq!(recordcount, 2);
    }

    #[test]
    fn test_convert_fastq_fasta() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let output_path = temp_file.path().to_str().unwrap();
        let result = convert_fastq_to_fasta("testdata/fastx/small.fastq", output_path);

        assert!(result.is_ok());
        assert!(temp_file.path().exists());
        let metadata = fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_convert_fastq_fasta_tauri() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let output_path = temp_file.path().to_str().unwrap();
        let result = convert_fastq_to_fasta_tauri("testdata/fastx/small.fastq", output_path);

        assert!(result.is_ok());
        assert!(temp_file.path().exists());
        let metadata = fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.len() > 0);
    }
}
