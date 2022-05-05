use crate::align::backtrace::{backtrace, AlignmentOutput};
use crate::align::band_2d::simple_stripes;
use crate::align::band_2d::Stripe;
use crate::align::score_matrix::{score_matrix, ScoreMatrixResult};
use crate::align::seed_alignment::seed_alignment;
use crate::cli::nextalign_cli::AlignPairwiseParams;
use crate::io::aa::Aa;
use crate::io::letter::Letter;
use crate::io::nuc::Nuc;
use crate::make_error;
use eyre::Report;
use log::trace;

fn align_pairwise<T: Letter<T>>(
  qry_seq: &[T],
  ref_seq: &[T],
  gap_open_close: &[i32],
  params: &AlignPairwiseParams,
  stripes: &[Stripe],
) -> Result<AlignmentOutput<T>, Report> {
  trace!("Align pairwise: started. Params: {params:?}");

  let max_indel = params.max_indel;

  let ScoreMatrixResult { scores, paths } = score_matrix(qry_seq, ref_seq, gap_open_close, stripes, params);

  Ok(backtrace(qry_seq, ref_seq, &scores, &paths))
}

pub fn align_nuc(
  qry_seq: &[Nuc],
  ref_seq: &[Nuc],
  gap_open_close: &[i32],
  params: &AlignPairwiseParams,
) -> Result<AlignmentOutput<Nuc>, Report> {
  let qry_len: usize = qry_seq.len();
  let min_len: usize = params.min_length;
  if qry_len < min_len {
    return make_error!(
      "Unable to align: sequence is too short. Details: sequence length: {qry_len}, min length allowed: {min_len}"
    );
  }

  let stripes = seed_alignment(qry_seq, ref_seq, params)?;

  let result = align_pairwise(qry_seq, ref_seq, gap_open_close, params, &stripes)?;

  trace!("Score: {}", result.alignment_score);

  Ok(result)
}

pub fn align_aa(
  qry_seq: &[Aa],
  ref_seq: &[Aa],
  gap_open_close: &[i32],
  params: &AlignPairwiseParams,
  band_width: usize,
  mean_shift: i32,
) -> Result<AlignmentOutput<Aa>, Report> {
  let stripes = simple_stripes(mean_shift, band_width, ref_seq.len(), qry_seq.len());

  align_pairwise(qry_seq, ref_seq, gap_open_close, params, &stripes)
}

#[cfg(test)]
mod tests {
  #![allow(clippy::needless_pass_by_value)]
  use std::fs;
  use std::path::PathBuf;

  // rstest fixtures are passed by value
  use super::*;
  use crate::align::gap_open::{get_gap_open_close_scores_codon_aware, GapScoreMap};
  use crate::gene::gene_map::GeneMap;
  use crate::io::nuc::{from_nuc_seq, to_nuc_seq};
  use eyre::Report;
  use pretty_assertions::assert_eq;
  use rstest::{fixture, rstest};

  struct Context {
    params: AlignPairwiseParams,
    gap_open_close: GapScoreMap,
  }

  #[fixture]
  fn ctx() -> Context {
    let params = AlignPairwiseParams {
      min_length: 3,
      ..AlignPairwiseParams::default()
    };

    let gene_map = GeneMap::new();

    let dummy_ref_seq = vec![Nuc::Gap; 100];
    let gap_open_close = get_gap_open_close_scores_codon_aware(&dummy_ref_seq, &gene_map, &params);

    Context { params, gap_open_close }
  }

  #[fixture]
  fn more_realistic_ctx() -> Context {
    let params = AlignPairwiseParams::default();
    let gene_map = GeneMap::new();

    let mut ref_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ref_path.push("test_data");
    ref_path.push("reference.fasta");
    let ref_seq = to_nuc_seq(&fs::read_to_string(ref_path).unwrap().trim()).unwrap();

    let gap_open_close = get_gap_open_close_scores_codon_aware(&ref_seq, &gene_map, &params);

    Context { params, gap_open_close }
  }

  #[rstest]
  fn aligns_identical(ctx: Context) -> Result<(), Report> {
    let qry_seq = to_nuc_seq("ACGCTCGCT")?;
    let ref_seq = to_nuc_seq("ACGCTCGCT")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_seq), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn pads_missing_left_single(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq( "CGCTCGCT")?;
    let ref_seq = to_nuc_seq("ACGCTCGCT")?;
    let qry_aln = to_nuc_seq("-CGCTCGCT")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn pads_missing_left(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq(   "CTCGCT")?;
    let ref_seq = to_nuc_seq("ACGCTCGCT")?;
    let qry_aln = to_nuc_seq("---CTCGCT")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn pads_missing_left_with_single_mismatch(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq(     "TCCAATCA")?;
    let ref_seq = to_nuc_seq("AACAAACCAACCA")?;
    let qry_aln = to_nuc_seq("-----TCCAATCA")?;
    //                                  ^

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn pads_missing_left_with_mismatches_adjacent(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq(     "TGTTACCTGCGC")?;
    let ref_seq = to_nuc_seq("AAGGTTTATACCTGCGC")?;
    let qry_aln = to_nuc_seq("-----TGTTACCTGCGC")?;
    //                              ^^

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn pads_missing_right(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq("ACGCTC"     )?;
    let ref_seq = to_nuc_seq("ACGCTCGCT")?;
    let qry_aln = to_nuc_seq("ACGCTC---")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn pads_missing_right_with_single_mismatch(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq("CCAATCAT"     )?;
    let ref_seq = to_nuc_seq("CCAACCAAACAAA")?;
    let qry_aln = to_nuc_seq("CCAATCAT-----")?;
    //                             ^

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn pads_missing_right_with_multiple_mismatches(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq("CCGATCAT"     )?;
    let ref_seq = to_nuc_seq("CCGACCAAACAAA")?;
    let qry_aln = to_nuc_seq("CCGATCAT-----")?;
    //                            ^  ^

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn handles_query_contained_in_ref(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq(   "ACGCTC"   )?;
    let ref_seq = to_nuc_seq("GCCACGCTCGCT")?;
    let qry_aln = to_nuc_seq("---ACGCTC---")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn handles_ref_contained_in_query(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq("GCCACGCTCGCT")?;
    let ref_seq = to_nuc_seq("ACGCTC")?;
    let ref_aln = to_nuc_seq("---ACGCTC---")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_aln), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_seq), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn adds_gaps_when_one_mismatch(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq(  "GCCACTCCCT")?;
    let ref_seq = to_nuc_seq("GCCACGCTCGCT")?;
    let qry_aln = to_nuc_seq("GCCA--CTCCCT")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    // assert_eq!(18, result.alignment_score);
    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn adds_gaps_in_ref_when_one_ambiguous_but_matching_char(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq("GCCACGCTCRCT")?;
    let ref_seq = to_nuc_seq("GCCACTCGCT")?;
    let ref_aln = to_nuc_seq("GCCA--CTCGCT")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_aln), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_seq), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn aligns_ambiguous_gap_placing(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq("ACATCTTC"   )?;
    let ref_seq = to_nuc_seq("ACATATACTTC")?;
    let qry_aln = to_nuc_seq("ACAT---CTTC")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_seq), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn aligns_ambiguous_gap_placing_case_reversed(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq("ACATATACTTG")?;
    let ref_seq = to_nuc_seq("ACATCTTG")?;
    let ref_aln = to_nuc_seq("ACAT---CTTG")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_aln), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_seq), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn aligns_minimal_overlap_qry_first(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let qry_seq = to_nuc_seq("AAAAAAAAAAAA")?;
    let ref_seq = to_nuc_seq("AAATTTTTTTTTT")?;
    let qry_aln = to_nuc_seq("AAAAAAAAAAAA----------")?;
    let ref_aln = to_nuc_seq("---------AAATTTTTTTTTT")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_aln), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn aligns_minimal_overlap_ref_first(ctx: Context) -> Result<(), Report> {
    #[rustfmt::skip]
    let ref_seq = to_nuc_seq("AAAAAAAAAAAA")?;
    let qry_seq = to_nuc_seq("AAATTTTTTTTTT")?;
    let ref_aln = to_nuc_seq("AAAAAAAAAAAA----------")?;
    let qry_aln = to_nuc_seq("---------AAATTTTTTTTTT")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_aln), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }

  #[rstest]
  fn aligns_real_large_sc2_qry_insertion_at_start(more_realistic_ctx: Context) -> Result<(), Report> {
    let mut test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_data_dir.push("test_data");
    let mut ref_path = test_data_dir.clone();
    ref_path.push("reference.fasta");
    let mut qry_path = test_data_dir.clone();
    qry_path.push("Hangzhou_ZJU_07_2020.fasta");
    let mut ref_aln_path = test_data_dir.clone();
    ref_aln_path.push("Hangzhou_ZJU_07_2020_ref_aligned.fasta");
    let mut qry_aln_path = test_data_dir.clone();
    qry_aln_path.push("Hangzhou_ZJU_07_2020_qry_aligned.fasta");
    let ref_seq = to_nuc_seq(&fs::read_to_string(ref_path)?.trim())?;
    let qry_seq = to_nuc_seq(&fs::read_to_string(qry_path)?.trim())?;
    let ref_aln = to_nuc_seq(&fs::read_to_string(ref_aln_path)?.trim())?;
    let qry_aln = to_nuc_seq(&fs::read_to_string(qry_aln_path)?.trim())?;

    let result = align_nuc(
      &qry_seq,
      &ref_seq,
      &more_realistic_ctx.gap_open_close,
      &more_realistic_ctx.params,
    )?;

    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    assert_eq!(from_nuc_seq(&ref_aln), from_nuc_seq(&result.ref_seq));
    Ok(())
  }

  #[rstest]
  #[rustfmt::skip]
  fn general_case(ctx: Context) -> Result<(), Report> {
    let ref_seq = to_nuc_seq("CTTGGAGGTTCCGTGGCTAGATAACAGAACATTCTTGGAATGCTGATCTTTATAAGCTCATGCGACACTTCGCATGGTGAGCCTTTGT"       )?;
    let qry_seq = to_nuc_seq("CTTGGAGGTTCCGTGGCTATAAAGATAACAGAACATTCTTGGAATGCTGATCAAGCTCATGGGACANNNNNCATGGTGGACAGCCTTTGT"     )?;
    let ref_aln = to_nuc_seq("CTTGGAGGTTCCGTG----GCTAGATAACAGAACATTCTTGGAATGCTGATCTTTATAAGCTCATGCGACACTTCGCATGGTG---AGCCTTTGT")?;
    let qry_aln = to_nuc_seq("CTTGGAGGTTCCGTGGCTATAAAGATAACAGAACATTCTTGGAATGCTGATC-----AAGCTCATGGGACANNNNNCATGGTGGACAGCCTTTGT")?;

    let result = align_nuc(&qry_seq, &ref_seq, &ctx.gap_open_close, &ctx.params)?;

    assert_eq!(from_nuc_seq(&ref_aln), from_nuc_seq(&result.ref_seq));
    assert_eq!(from_nuc_seq(&qry_aln), from_nuc_seq(&result.qry_seq));
    Ok(())
  }
}
