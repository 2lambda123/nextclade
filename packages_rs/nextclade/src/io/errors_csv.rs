use crate::io::gene_map::GeneMap;
use crate::io::csv::{CsvStructFileWriter, CsvStructWriter};
use crate::io::nextclade_csv::{format_aa_warnings, format_failed_genes};
use crate::translate::translate_genes::Translation;
use crate::types::outputs::PeptideWarning;
use crate::utils::error::report_to_string;
use eyre::Report;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorCsvEntry<'a, 'b> {
  pub seq_name: &'a str,
  pub errors: &'a str,
  pub warnings: &'b str,
  pub failed_genes: &'b str,
}

/// Writes errors.csv file
pub struct ErrorsCsvWriter<'a> {
  gene_map: &'a GeneMap,
  writer: CsvStructFileWriter,
}

impl<'a> ErrorsCsvWriter<'a> {
  pub fn new(gene_map: &'a GeneMap, filepath: impl AsRef<Path>) -> Result<Self, Report> {
    Ok(Self {
      gene_map,
      writer: CsvStructFileWriter::new(filepath.as_ref(), b',')?,
    })
  }

  /// Writes one row into errors.csv file for the case of nuc alignment error
  pub fn write_nuc_error(&mut self, seq_name: &str, message: &str) -> Result<(), Report> {
    self.writer.write(&ErrorCsvEntry {
      seq_name,
      errors: message,
      warnings: "",
      failed_genes: "",
    })
  }

  /// Writes one row into errors.csv file for the case of aa alignment errors
  pub fn write_aa_errors(
    &mut self,
    seq_name: &str,
    warnings: &[PeptideWarning],
    failed_genes: &[String],
  ) -> Result<(), Report> {
    let warnings = &warnings.iter().map(|PeptideWarning { warning, .. }| warning).join(";");
    let failed_genes = &format_failed_genes(failed_genes, ";");
    self.writer.write(&ErrorCsvEntry {
      seq_name,
      errors: "",
      warnings,
      failed_genes,
    })
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorsFromWeb {
  seq_name: String,
  errors: String,
  warnings: Vec<PeptideWarning>,
  failed_genes: Vec<String>,
}

pub fn errors_to_csv_string(errors: &[ErrorsFromWeb]) -> Result<String, Report> {
  let mut writer = CsvStructWriter::new(Vec::<u8>::new(), b',')?;

  for error in errors {
    let warnings = &error
      .warnings
      .iter()
      .map(|PeptideWarning { warning, .. }| warning)
      .join(";");

    let failed_genes = &format_failed_genes(&error.failed_genes, ";");

    writer.write(&ErrorCsvEntry {
      seq_name: &error.seq_name,
      errors: &error.errors,
      warnings,
      failed_genes,
    })?;
  }

  Ok(String::from_utf8(writer.into_inner()?)?)
}
