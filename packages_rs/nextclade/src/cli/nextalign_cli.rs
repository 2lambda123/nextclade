use crate::io::fs::basename;
use crate::utils::global_init::setup_logger;
use clap::{AppSettings, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{generate, Generator, Shell};
use clap_complete_fig::Fig;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use eyre::{eyre, Report, WrapErr};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::LevelFilter;
use std::env::current_dir;
use std::fmt::Debug;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

lazy_static! {
  static ref SHELLS: &'static [&'static str] = &["bash", "elvish", "fish", "fig", "powershell", "zsh"];
  static ref VERBOSITIES: &'static [&'static str] = &["off", "error", "warn", "info", "debug", "trace"];
}

#[derive(Parser, Debug)]
#[clap(name = "Nextalign", trailing_var_arg = true)]
#[clap(author, version)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
#[clap(verbatim_doc_comment)]
/// Viral sequence alignment and translation.
///
/// Nextalign is a part of Nextstrain project: https://nextstrain.org
///
/// Documentation: https://docs.nextstrain.org/projects/nextclade
/// Nextclade Web: https://clades.nextstrain.org
/// Publication:   https://doi.org/10.21105/joss.03773
pub struct NextalignArgs {
  #[clap(subcommand)]
  pub command: Option<NextalignCommands>,

  /// Make output more quiet or more verbose
  #[clap(flatten)]
  pub verbose: Verbosity<WarnLevel>,

  /// Set verbosity level
  #[clap(long, global = true, conflicts_with = "verbose", conflicts_with = "silent", possible_values(VERBOSITIES.iter()))]
  pub verbosity: Option<log::LevelFilter>,

  /// Disable all console output. Same as --verbosity=off
  #[clap(long, global = true, conflicts_with = "verbose", conflicts_with = "verbosity")]
  pub silent: bool,
}

#[derive(Subcommand, Debug)]
#[clap(verbatim_doc_comment)]
pub enum NextalignCommands {
  /// Generate shell completions.
  ///
  /// This will print the completions file contents to the console. Refer to your shell's documentation on how to install the completions.
  ///
  /// Example for Ubuntu Linux:
  ///
  ///    nextalign completions bash > ~/.local/share/bash-completion/nextalign
  ///
  Completions {
    /// Name of the shell to generate appropriate completions
    #[clap(value_name = "SHELL", default_value_t = String::from("bash"), possible_values(SHELLS.iter()))]
    shell: String,
  },

  /// Run alignment and translation.
  Run(Box<NextalignRunArgs>),
}

#[derive(Parser, Debug)]
pub struct NextalignRunArgs {
  /// Path to a FASTA file with input sequences"
  #[clap(long, short = 'i', alias("sequences"))]
  #[clap(value_hint = ValueHint::FilePath)]
  pub input_fasta: PathBuf,

  /// Path to a FASTA file containing reference sequence.This file is expected to contain exactly 1 sequence.
  #[clap(long, short = 'r', alias("reference"))]
  #[clap(value_hint = ValueHint::FilePath)]
  pub input_ref: PathBuf,

  /// List of genes to translate.
  ///
  /// If not supplied or empty, sequence will not be translated. If non-empty, should contain a coma-separated list of gene names.
  ///
  /// Parameters `--genes` and `--genemap` should be either both specified or both omitted.
  #[clap(
    long,
    short = 'g',
    takes_value = true,
    multiple_values = true,
    use_value_delimiter = true
  )]
  #[clap(value_hint = ValueHint::FilePath)]
  pub genes: Option<Vec<String>>,

  #[clap(long, short = 'm')]
  #[clap(value_hint = ValueHint::FilePath)]
  /// Path to a GFF3 file containing custom gene map.
  ///
  /// If not supplied, sequence will not be translated.
  ///
  /// Parameters `--genes` and `--genemap` should be either both specified or both omitted.
  ///
  /// Learn more about Generic Feature Format Version 3 (GFF3):
  /// https://github.com/The-Sequence-Ontology/Specifications/blob/master/gff3.md",
  pub genemap: Option<PathBuf>,

  /// Write output files to this directory.
  ///
  /// The base filename can be set using `--output-basename` flag. The paths can be overridden on a per-file basis using `--output-*` flags.
  ///
  /// If the required directory tree does not exist, it will be created.
  #[clap(long, short = 'd')]
  #[clap(value_hint = ValueHint::DirPath)]
  pub output_dir: Option<PathBuf>,

  /// Set the base filename to use for output files.
  ///
  /// To be used together with `--output-dir` flag. By default uses the filename of the sequences file (provided with `--input-fasta`). The paths can be overridden on a per-file basis using `--output-*` flags.
  #[clap(long, short = 'n')]
  pub output_basename: Option<String>,

  /// Whether to include aligned reference nucleotide sequence into output nucleotide sequence FASTA file and reference peptides into output peptide FASTA files.
  #[clap(long)]
  pub include_reference: Option<bool>,

  /// Path to output FASTA file with aligned sequences.
  ///
  /// Overrides paths given with `--output-dir` and `--output-basename`.
  ///
  /// If the required directory tree does not exist, it will be created.
  #[clap(long, short = 'o')]
  #[clap(value_hint = ValueHint::AnyPath)]
  pub output_fasta: Option<PathBuf>,

  /// Path to output CSV file with stripped insertions data.
  ///
  /// Overrides paths given with `--output-dir` and `--output-basename`.
  ///
  /// If the required directory tree does not exist, it will be created.",
  #[clap(long, short = 'I')]
  #[clap(value_hint = ValueHint::AnyPath)]
  pub output_insertions: Option<PathBuf>,

  /// Path to output CSV file containing errors and warnings occurred during processing
  ///
  /// Overrides paths given with `--output-dir` and `--output-basename`).
  ///
  /// If the required directory tree does not exist, it will be created
  #[clap(long, short = 'e')]
  #[clap(value_hint = ValueHint::AnyPath)]
  pub output_errors: Option<PathBuf>,

  /// Number of processing jobs. If not specified, all available CPU threads will be used.
  #[clap(long, short)]
  pub jobs: Option<usize>,

  /// Emit output sequences in-order.
  ///
  /// With this flag the program will wait for results from the previous sequences to be written to the output files before writing the results of the next sequences, preserving the same order as in the input file. Due to variable sequence processing times, this might introduce unnecessary waiting times, but ensures that the resulting sequences are written in the same order as they occur in the inputs (except for sequences which have errors).
  /// By default, without this flag, processing might happen out of order, which is faster, due to the elimination of waiting, but might also lead to results written out of order - the order of results is not specified and depends on thread scheduling and processing times of individual sequences.
  ///
  /// This option is only relevant when `--jobs` is greater than 1 or is omitted.
  ///
  /// Note: the sequences which trigger errors during processing will be omitted from outputs, regardless of this flag.
  #[clap(long)]
  pub in_order: bool,
}

fn generate_completions(shell: &str) -> Result<(), Report> {
  let mut command = NextalignArgs::command();

  if shell.to_lowercase() == "fig" {
    generate(Fig, &mut command, "nextalign", &mut io::stdout());
    return Ok(());
  }

  let generator =
    Shell::from_str(&shell.to_lowercase()).map_err(|err| eyre!("{}: Possible values: {}", err, SHELLS.join(", ")))?;

  let bin_name = command.get_name().to_owned();

  generate(generator, &mut command, bin_name, &mut io::stdout());

  Ok(())
}

/// Get output filenames provided by user or, if not provided, create filenames based on input fasta
pub fn get_output_filenames(run_args: &mut NextalignRunArgs) -> Result<(), Report> {
  let NextalignRunArgs { input_fasta, .. } = run_args;

  let basename = run_args.output_basename.get_or_insert(basename(&input_fasta)?);

  let output_dir = run_args
    .output_dir
    .get_or_insert(current_dir().wrap_err("When getting current working directory")?);

  run_args
    .output_fasta
    .get_or_insert(output_dir.join(&basename).with_extension("aligned.fasta"));

  run_args
    .output_insertions
    .get_or_insert(output_dir.join(&basename).with_extension("insertions.csv"));

  run_args
    .output_errors
    .get_or_insert(output_dir.join(&basename).with_extension("errors.csv"));

  Ok(())
}

pub fn nextalign_parse_cli_args() -> Result<NextalignArgs, Report> {
  let mut args = NextalignArgs::parse();

  // --verbosity=<level> and --silent take priority over -v and -q
  let filter_level = if args.silent {
    LevelFilter::Off
  } else {
    match args.verbosity {
      None => args.verbose.log_level_filter(),
      Some(verbosity) => verbosity,
    }
  };

  setup_logger(filter_level);

  match &mut args.command {
    Some(NextalignCommands::Completions { shell }) => {
      generate_completions(shell).wrap_err_with(|| format!("When generating completions for shell '{shell}'"))?;
    }
    Some(NextalignCommands::Run { 0: ref mut run_args }) => {
      get_output_filenames(run_args).wrap_err("When deducing output filenames")?;
    }
    _ => {}
  }

  Ok(args)
}