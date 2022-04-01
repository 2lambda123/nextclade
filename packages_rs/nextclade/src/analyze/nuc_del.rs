use crate::analyze::nuc_sub::NucSub;
use crate::io::letter::Letter;
use crate::io::nuc::Nuc;
use crate::utils::range::Range;
use eyre::Report;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NucDel {
  pub start: usize,
  pub length: usize,
}

impl NucDel {
  #[inline]
  pub fn end(&self) -> usize {
    self.start + self.length
  }

  #[inline]
  pub fn to_range(&self) -> Range {
    Range {
      begin: self.start,
      end: self.end(),
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NucDelMinimal {
  #[serde(rename = "ref")]
  pub reff: Nuc,
  pub pos: usize,
}

impl NucDelMinimal {
  /// Converts deletion to substitution to Gap
  #[inline]
  pub fn to_sub(&self) -> NucSub {
    NucSub {
      reff: self.reff,
      pos: self.pos,
      qry: Nuc::Gap,
    }
  }
}

/// Order deletions by position, then ref character
impl Ord for NucDelMinimal {
  fn cmp(&self, other: &Self) -> Ordering {
    (self.pos, self.reff).cmp(&(other.pos, other.reff))
  }
}

impl PartialOrd for NucDelMinimal {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
