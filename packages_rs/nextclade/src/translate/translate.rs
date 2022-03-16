#![allow(clippy::integer_division)]

use crate::align::align::AlignPairwiseParams;
use crate::gene::gene::Gene;
use crate::io::aa::Aa;
use crate::io::nuc::Nuc;

use crate::translate::peptide::Peptide;
use eyre::Report;

pub fn decode(triplet: &[Nuc]) -> Aa {
  match *triplet {
    [Nuc::GAP, Nuc::GAP, Nuc::GAP] => Aa::GAP,
    [Nuc::A, Nuc::A, Nuc::A] => Aa::K,
    [Nuc::A, Nuc::A, Nuc::C] => Aa::N,
    [Nuc::A, Nuc::A, Nuc::G] => Aa::K,
    [Nuc::A, Nuc::A, Nuc::T] => Aa::N,
    [Nuc::A, Nuc::C, Nuc::A] => Aa::T,
    [Nuc::A, Nuc::C, Nuc::C] => Aa::T,
    [Nuc::A, Nuc::C, Nuc::G] => Aa::T,
    [Nuc::A, Nuc::C, Nuc::T] => Aa::T,
    [Nuc::A, Nuc::G, Nuc::A] => Aa::R,
    [Nuc::A, Nuc::G, Nuc::C] => Aa::S,
    [Nuc::A, Nuc::G, Nuc::G] => Aa::R,
    [Nuc::A, Nuc::G, Nuc::T] => Aa::S,
    [Nuc::A, Nuc::T, Nuc::A] => Aa::I,
    [Nuc::A, Nuc::T, Nuc::C] => Aa::I,
    [Nuc::A, Nuc::T, Nuc::G] => Aa::M,
    [Nuc::A, Nuc::T, Nuc::T] => Aa::I,
    [Nuc::C, Nuc::A, Nuc::A] => Aa::Q,
    [Nuc::C, Nuc::A, Nuc::C] => Aa::H,
    [Nuc::C, Nuc::A, Nuc::G] => Aa::Q,
    [Nuc::C, Nuc::A, Nuc::T] => Aa::H,
    [Nuc::C, Nuc::C, Nuc::A] => Aa::P,
    [Nuc::C, Nuc::C, Nuc::C] => Aa::P,
    [Nuc::C, Nuc::C, Nuc::G] => Aa::P,
    [Nuc::C, Nuc::C, Nuc::T] => Aa::P,
    [Nuc::C, Nuc::G, Nuc::A] => Aa::R,
    [Nuc::C, Nuc::G, Nuc::C] => Aa::R,
    [Nuc::C, Nuc::G, Nuc::G] => Aa::R,
    [Nuc::C, Nuc::G, Nuc::T] => Aa::R,
    [Nuc::C, Nuc::T, Nuc::A] => Aa::L,
    [Nuc::C, Nuc::T, Nuc::C] => Aa::L,
    [Nuc::C, Nuc::T, Nuc::G] => Aa::L,
    [Nuc::C, Nuc::T, Nuc::T] => Aa::L,
    [Nuc::G, Nuc::A, Nuc::A] => Aa::E,
    [Nuc::G, Nuc::A, Nuc::C] => Aa::D,
    [Nuc::G, Nuc::A, Nuc::G] => Aa::E,
    [Nuc::G, Nuc::A, Nuc::T] => Aa::D,
    [Nuc::G, Nuc::C, Nuc::A] => Aa::A,
    [Nuc::G, Nuc::C, Nuc::C] => Aa::A,
    [Nuc::G, Nuc::C, Nuc::G] => Aa::A,
    [Nuc::G, Nuc::C, Nuc::T] => Aa::A,
    [Nuc::G, Nuc::G, Nuc::A] => Aa::G,
    [Nuc::G, Nuc::G, Nuc::C] => Aa::G,
    [Nuc::G, Nuc::G, Nuc::G] => Aa::G,
    [Nuc::G, Nuc::G, Nuc::T] => Aa::G,
    [Nuc::G, Nuc::T, Nuc::A] => Aa::V,
    [Nuc::G, Nuc::T, Nuc::C] => Aa::V,
    [Nuc::G, Nuc::T, Nuc::G] => Aa::V,
    [Nuc::G, Nuc::T, Nuc::T] => Aa::V,
    [Nuc::T, Nuc::A, Nuc::A] => Aa::STOP,
    [Nuc::T, Nuc::A, Nuc::C] => Aa::Y,
    [Nuc::T, Nuc::A, Nuc::G] => Aa::STOP,
    [Nuc::T, Nuc::A, Nuc::T] => Aa::Y,
    [Nuc::T, Nuc::C, Nuc::A] => Aa::S,
    [Nuc::T, Nuc::C, Nuc::C] => Aa::S,
    [Nuc::T, Nuc::C, Nuc::G] => Aa::S,
    [Nuc::T, Nuc::C, Nuc::T] => Aa::S,
    [Nuc::T, Nuc::G, Nuc::A] => Aa::STOP,
    [Nuc::T, Nuc::G, Nuc::C] => Aa::C,
    [Nuc::T, Nuc::G, Nuc::G] => Aa::W,
    [Nuc::T, Nuc::G, Nuc::T] => Aa::C,
    [Nuc::T, Nuc::T, Nuc::A] => Aa::L,
    [Nuc::T, Nuc::T, Nuc::C] => Aa::F,
    [Nuc::T, Nuc::T, Nuc::G] => Aa::L,
    [Nuc::T, Nuc::T, Nuc::T] => Aa::F,
    _ => Aa::X,
  }
}

/// Translates a nucleotide sequence of a gene into the corresponding aminoacid sequence (peptide)
/// NOTE: we accept gene sequence by value here to avoid copying (it should be moved) and then process it in-place
pub fn translate(gene_nuc_seq: &[Nuc], gene: &Gene, params: &AlignPairwiseParams) -> Result<Peptide, Report> {
  // NOTE: rounds the result to the multiple of 3 (floor) so that translation does not overrun the buffer
  let peptide_length = gene_nuc_seq.len() / 3;

  let mut peptide = Vec::<Aa>::with_capacity(peptide_length);
  for i_aa in 0..peptide_length {
    let i_nuc = i_aa * 3;
    let triplet: &[Nuc] = &gene_nuc_seq[i_nuc..(i_nuc + 3)];
    let aminoacid = decode(triplet);
    peptide.push(aminoacid);
    if !params.translatePastStop && aminoacid == Aa::STOP {
      break;
    }
  }
  peptide.shrink_to_fit();

  Ok(Peptide {
    gene_name: gene.gene_name.clone(),
    seq: peptide,
  })
}
