use clap::{Parser, ValueHint};
use ctor::ctor;
use eyre::Report;
use log::LevelFilter;
use nextclade::analyze::pcr_primers::PcrPrimer;
use nextclade::analyze::virus_properties::{PhenotypeAttrDesc, VirusProperties};
use nextclade::gene::gene_map::GeneMap;
use nextclade::io::dataset::{DatasetTagJson, DatasetsIndexJson};
use nextclade::io::errors_csv::ErrorsFromWeb;
use nextclade::io::fasta::FastaRecord;
use nextclade::io::file::create_file_or_stdout;
use nextclade::io::json::json_write_impl;
use nextclade::io::nextclade_csv::CsvColumnConfig;
use nextclade::qc::qc_config::QcConfig;
use nextclade::qc::qc_run::QcResult;
use nextclade::translate::translate_genes::Translation;
use nextclade::tree::tree::{AuspiceTree, CladeNodeAttrKeyDesc};
use nextclade::types::outputs::{NextcladeErrorOutputs, NextcladeOutputs};
use nextclade::utils::global_init::global_init;
use nextclade::utils::global_init::setup_logger;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::{stdout, Write};
use std::path::PathBuf;

#[ctor]
fn init() {
  global_init();
}

#[derive(Parser, Debug)]
#[clap(name = "generate_jsonschema")]
#[clap(author, version)]
#[clap(verbatim_doc_comment)]
pub struct FeaturemapArgs {
  /// Path to output directory
  #[clap(long, short = 'o')]
  #[clap(value_hint = ValueHint::DirPath)]
  pub output: Option<PathBuf>,
}

fn write_jsonschema<T: JsonSchema>(output: &Option<PathBuf>) -> Result<(), Report> {
  let writer: Box<dyn Write + Send> = match output {
    None => Box::new(stdout()),
    Some(output) => {
      let filename = format!("{}.json", T::schema_name());
      create_file_or_stdout(&output.join(filename))?
    }
  };

  let schema = schema_for!(T);
  json_write_impl(writer, &schema)
}

fn main() -> Result<(), Report> {
  let args = FeaturemapArgs::parse();
  setup_logger(LevelFilter::Warn);

  write_jsonschema::<_SchemaRoot>(&args.output)?;

  Ok(())
}

// Dummy struct containing the types we want to expose (recursively).
//
// The doc comment will appear in the schema file.
//
/// AUTOGENERATED! DO NOT EDIT! This JSON schema file is generated automatically from Rust types.
/// The topmost schema definition is a dummy container for the types we want to expose. Disregard
/// it. Instead, See the actual types in the `definitions` property of JSON schema.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct _SchemaRoot {
  _1: GeneMap,
  _2: Translation,
  _3: AuspiceTree,
  _4: QcConfig,
  _5: QcResult,
  _6: PcrPrimer,
  _7: NextcladeOutputs,
  _8: DatasetsIndexJson,
  _9: CsvColumnConfig,
  _10: NextcladeErrorOutputs,
  _11: ErrorsFromWeb,
  _12: VirusProperties,
  _13: CladeNodeAttrKeyDesc,
  _14: PhenotypeAttrDesc,
  _15: FastaRecord,
  _16: DatasetTagJson,
}