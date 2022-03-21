use crate::align::align::{align_aa, AlignPairwiseParams};
use crate::align::remove_gaps::remove_gaps_in_place;
use crate::align::strip_insertions::{strip_insertions, Insertion};
use crate::analyze::count_gaps::GapCounts;
use crate::gene::gene::Gene;
use crate::gene::gene_map::GeneMap;
use crate::io::aa::Aa;
use crate::io::letter::Letter;
use crate::io::nuc::Nuc;
use crate::translate::complement::reverse_complement_in_place;
use crate::translate::coord_map::CoordMap;
use crate::translate::frame_shifts_detect::frame_shifts_detect;
use crate::translate::frame_shifts_translate::{frame_shifts_translate, FrameShift};
use crate::translate::peptide::{Peptide, PeptideMap};
use crate::translate::translate::translate;
use crate::utils::range::Range;
use crate::{make_error, make_internal_report};
use eyre::Report;
use itertools::Itertools;
use std::ops::Range as StdRange;

/// Results of the translation
pub struct Translation {
  pub gene_name: String,
  pub seq: Vec<Aa>,
  pub insertions: Vec<Insertion<Aa>>,
  pub frame_shifts: Vec<FrameShift>,
}

/// Extracts nucleotide sequence of a gene
pub fn extract_gene(full_aln_seq: &[Nuc], gene: &Gene, coord_map: &CoordMap) -> Vec<Nuc> {
  let &Gene { start, end, .. } = gene;

  // Gene map contains gene range in reference coordinates (like in ref sequence)
  let gene_range_ref = Range { begin: start, end };

  // ...but we are extracting from aligned sequence, so we need to convert it to alignment coordinates (like in aligned sequences)
  let gene_range_aln = coord_map.ref_to_aln(&gene_range_ref);

  full_aln_seq[StdRange::from(gene_range_aln)].to_vec()
}

/// Results of the aminoacid alignment parameters estimation
pub struct PeptideAlignmentParams {
  band_width: usize,
  mean_shift: i32,
}

/// Deduces aminoacid alignment params from nucleotide alignment.
/// Gap counts from nucleotide alignment are used to estimate rough parameters. This is by contrast to nucleotide
/// alignment, where these parameters are estimated using seed matching.
#[allow(clippy::integer_division)]
pub fn calculate_aa_alignment_params(qry_gaps: &GapCounts, ref_gaps: &GapCounts) -> PeptideAlignmentParams {
  const BASE_BAND_WIDTH: usize = 5; // An arbitrary magic number to give some additional room for alignment
  let band_width = (qry_gaps.internal + ref_gaps.internal) / 3 + BASE_BAND_WIDTH;
  let mean_shift =
    (qry_gaps.leading as i32 - ref_gaps.leading as i32) / 3 + (qry_gaps.internal as i32 - ref_gaps.internal as i32) / 6;
  PeptideAlignmentParams { band_width, mean_shift }
}

/// Replaces first and second gap in a not-all-gap triplet with `N`
pub fn protect_codon_in_place(triplet: &mut [Nuc]) {
  if triplet[0].is_gap() {
    triplet[0] = Nuc::N;

    if triplet[1].is_gap() {
      triplet[1] = Nuc::N;
    }
  }
}

/// Finds the first non-GAP nucleotide and replaces GAPs in the corresponding triplet with Ns,
/// so that it's not getting stripped during subsequent gap stripping. This is to ensure the first codon is complete
/// and no shift is introduced during subsequent gap-stripping
pub fn protect_first_codon_in_place(seq: &mut [Nuc]) {
  // TODO: this is a very C-like function. Try to replace the indexed loop with something more idiomatic
  let length = seq.len();
  let end = length - (length % 3);
  for i in 0..end {
    if !seq[i].is_gap() {
      let triplet_begin = i - (i % 3);
      let triplet = &mut seq[triplet_begin..triplet_begin + 3];
      protect_codon_in_place(triplet);
      break;
    }
  }
}

/// Masks frame shifts in the nucleotide sequence with `N`
pub fn mask_nuc_frame_shifts_in_place(seq: &mut [Nuc], frame_shifts: &[FrameShift]) {
  // TODO: this is a very C-like function. Try to replace the indexed loop with something more idiomatic
  for frame_shift in frame_shifts {
    let mut current = frame_shift.nuc_rel.begin;
    let end = frame_shift.nuc_rel.end;
    while current < end {
      if !seq[current].is_gap() {
        seq[current] = Nuc::N;
      }
      current += 1;
    }
  }
}

pub fn fill_range_inplace(seq: &mut [Aa], range: &Range, letter: Aa) {
  // TODO: this is a very C-like function. Try to replace the indexed loop with something more idiomatic
  let mut current = range.begin;
  let end = range.end;
  while current < end {
    seq[current] = letter;
    current += 1;
  }
}

/// Masks gaps in frame-shifted regions of the peptide.
/// The frame-shifted region is likely misaligned, so the gaps added during peptide alignment don't make sense
/// and we cover them with `X`.
pub fn mask_peptide_frame_shifts_in_place(seq: &mut [Aa], frame_shifts: &[FrameShift]) {
  for frame_shift in frame_shifts {
    fill_range_inplace(seq, &frame_shift.gaps_leading.codon, Aa::GAP);
    fill_range_inplace(seq, &frame_shift.codon, Aa::X);
    fill_range_inplace(seq, &frame_shift.gaps_trailing.codon, Aa::GAP);
  }
}

pub fn translate_gene(
  qry_seq: &[Nuc],
  ref_seq: &[Nuc],
  gene: &Gene,
  ref_peptide: &Peptide,
  gap_open_close_aa: &[i32],
  coord_map: &CoordMap,
  params: &AlignPairwiseParams,
) -> Result<Translation, Report> {
  let mut ref_gene_seq = extract_gene(ref_seq, gene, coord_map);
  let mut qry_gene_seq = extract_gene(qry_seq, gene, coord_map);

  // Reverse strands should be reverse-complemented first
  if gene.strand == "-" {
    reverse_complement_in_place(&mut ref_gene_seq);
    reverse_complement_in_place(&mut qry_gene_seq);
  }

  let ref_gaps = GapCounts::new(&ref_gene_seq);
  let qry_gaps = GapCounts::new(&qry_gene_seq);

  if ref_gene_seq.is_empty() || qry_gaps.is_all_gaps() {
    return make_error!(
      "When processing gene \"{}\": The extracted gene sequence is empty or consists entirely from gaps",
      &gene.gene_name
    );
  }

  // Make sure subsequent gap stripping does not introduce frame shift
  protect_first_codon_in_place(&mut ref_gene_seq);
  protect_first_codon_in_place(&mut qry_gene_seq);

  // NOTE: frame shift detection should be performed on unstripped genes
  let nuc_rel_frame_shifts = frame_shifts_detect(&qry_gene_seq, &ref_gene_seq);
  let frame_shifts = frame_shifts_translate(&nuc_rel_frame_shifts, &qry_gene_seq, coord_map, gene);

  mask_nuc_frame_shifts_in_place(&mut qry_gene_seq, &frame_shifts);

  // Strip all GAP characters to "forget" gaps introduced during alignment
  remove_gaps_in_place(&mut qry_gene_seq);

  let query_peptide = translate(&qry_gene_seq, gene, params)?;

  // Instead of performing seed matching, like we do for nucleotide alignment, here we estimate parameters
  // by counting gaps in the aligned nucleotide sequences;
  let PeptideAlignmentParams { band_width, mean_shift } = calculate_aa_alignment_params(&qry_gaps, &ref_gaps);

  let alignment = align_aa(
    &query_peptide.seq,
    &ref_peptide.seq,
    gap_open_close_aa,
    params,
    band_width,
    mean_shift,
  )?;

  let mut stripped = strip_insertions(&alignment.qry_seq, &alignment.ref_seq);

  mask_peptide_frame_shifts_in_place(&mut stripped.qry_seq, &frame_shifts);

  Ok(Translation {
    gene_name: gene.gene_name.clone(),
    seq: stripped.qry_seq,
    insertions: stripped.insertions,
    frame_shifts,
  })
}

pub fn translate_genes(
  qry_seq: &[Nuc],
  ref_seq: &[Nuc],
  ref_peptides: &PeptideMap,
  gene_map: &GeneMap,
  gap_open_close_aa: &[i32],
  params: &AlignPairwiseParams,
) -> Vec<Result<Translation, Report>> {
  let coord_map = CoordMap::new(ref_seq);

  gene_map
    .iter()
    .map(|(_, gene)| -> Result<_, Report> {
      let ref_peptide = ref_peptides.get(&gene.gene_name).ok_or(make_internal_report!(
        "Reference peptide not found for gene {}",
        &gene.gene_name
      ))?;

      translate_gene(
        qry_seq,
        ref_seq,
        gene,
        ref_peptide,
        gap_open_close_aa,
        &coord_map,
        params,
      )
    })
    .collect_vec()
}