use crate::align::params::AlignPairwiseParams;
use crate::io::gene_map::GeneMap;
use crate::io::nuc::Nuc;

pub type GapScoreMap = Vec<i32>;

pub fn get_gap_open_close_scores_flat(ref_seq: &[Nuc], params: &AlignPairwiseParams) -> GapScoreMap {
  let value = params.penalty_gap_open;
  let len = ref_seq.len() + 2;
  vec![value; len]
}

pub fn get_gap_open_close_scores_codon_aware(
  ref_seq: &[Nuc],
  gene_map: &GeneMap,
  params: &AlignPairwiseParams,
) -> GapScoreMap {
  let mut gap_open_close = get_gap_open_close_scores_flat(ref_seq, params);
  for (_, gene) in gene_map.iter() {
    for i in (gene.start..gene.end).step_by(3) {
      gap_open_close[i] = params.penalty_gap_open_in_frame;
      gap_open_close[i + 1] = params.penalty_gap_open_out_of_frame;
      gap_open_close[i + 2] = params.penalty_gap_open_out_of_frame;
    }
  }
  gap_open_close
}
