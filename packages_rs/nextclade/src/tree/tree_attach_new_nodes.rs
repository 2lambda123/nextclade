use crate::analyze::aa_del::AaDelMinimal;
use crate::analyze::aa_sub::AaSubMinimal;
use crate::analyze::find_private_aa_mutations::PrivateAaMutations;
use crate::analyze::find_private_nuc_mutations::PrivateNucMutations;
use crate::analyze::nuc_del::NucDelMinimal;
use crate::analyze::nuc_sub::NucSub;
use crate::io::nextclade_csv::{
  format_failed_genes, format_missings, format_non_acgtns, format_nuc_deletions, format_pcr_primer_changes,
};
use crate::tree::tree::{
  AuspiceTree, AuspiceTreeNode, TreeBranchAttrs, TreeNodeAttr, TreeNodeAttrs, TreeNodeTempData, AUSPICE_UNKNOWN_VALUE,
};
use crate::types::outputs::NextcladeOutputs;
use crate::utils::collections::concat_to_vec;
use assert2::__assert2_impl::print;
use itertools::Itertools;
use serde_json::json;
use crate::tree::tree_builder::{calculate_distance_matrix, build_undirected_subtree, build_directed_subtree, add_mutations_to_vertices};
use crate::{extract_enum_value, tree::tree_builder::{NodeType, NewInternalNode, NewSeqNode, TreeNode, Graph, InternalMutations}};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use crate::io::nuc::Nuc;
use crate::translate::translate_genes::TranslationMap;
use crate::io::gene_map::GeneMap;
use crate::analyze::virus_properties::{VirusProperties, MutationLabelMaps};
use crate::utils::range::Range;
use crate::analyze::find_private_aa_mutations::find_private_aa_mutations;
use crate::analyze::find_private_nuc_mutations::find_private_nuc_mutations;
use crate::analyze::divergence::calculate_divergence;
use crate::tree::tree_preprocess::{map_aa_muts, map_nuc_muts};
use super::tree::DivergenceUnits;
use crate::io::aa::Aa;
use crate::io::letter::Letter;

pub fn tree_attach_new_nodes_in_place(tree: &mut AuspiceTree, results: &[NextcladeOutputs]) {
  tree_attach_new_nodes_impl_in_place_recursive(&mut tree.tree, results);
}

fn tree_attach_new_nodes_impl_in_place_recursive(node: &mut AuspiceTreeNode, results: &[NextcladeOutputs]) {
  // Attach only to a reference node.
  // If it's not a reference node, we can stop here, because there can be no reference nodes down the tree.
  if !node.tmp.is_ref_node {
    return;
  }

  for child in &mut node.children {
    tree_attach_new_nodes_impl_in_place_recursive(child, results);
  }

  // Look for a query sample result for which this node was decided to be nearest
  for result in results {
    if node.tmp.id == result.nearest_node_id {
      attach_new_node(node, result);
    }
  }
}

pub fn tree_attach_new_nodes_in_place_subtree(tree: &mut AuspiceTree, results: &[NextcladeOutputs], ref_seq: &[Nuc], ref_peptides: &TranslationMap, gene_map: &GeneMap, virus_properties: &VirusProperties) {
  let mut pos_to_attach = HashMap::<usize,Vec<usize>>::new();
  let mut i = 0;
  for result in results {
    let vec = Vec::new();
    let pos = pos_to_attach.entry(result.nearest_node_id).or_insert(vec);
    pos.push(i);
    i += 1;
  }
  let div_units = &tree.tmp.divergence_units;
  tree_attach_new_nodes_impl_in_place_recursive_subtree(&mut tree.tree, results, &pos_to_attach, ref_seq, ref_peptides, gene_map, virus_properties, div_units);
}

fn tree_attach_new_nodes_impl_in_place_recursive_subtree(node: &mut AuspiceTreeNode, results: &[NextcladeOutputs], attachment_positions: &HashMap<usize, Vec<usize>>, ref_seq: &[Nuc], ref_peptides: &TranslationMap, gene_map: &GeneMap, virus_properties: &VirusProperties, div_units: &DivergenceUnits) {
  // Attach only to a reference node.
  // If it's not a reference node, we can stop here, because there can be no reference nodes down the tree.
  if !node.tmp.is_ref_node {
    return;
  }

  for child in &mut node.children {
    tree_attach_new_nodes_impl_in_place_recursive_subtree(child, results, attachment_positions, ref_seq, ref_peptides, gene_map, virus_properties, div_units);
  }

  // Look for a query sample result for which this node was decided to be nearest
  let vec = attachment_positions.get(&node.tmp.id);
  if vec.is_some(){
    let unwrapped_vec = vec.unwrap();
    if unwrapped_vec.len() >2{
      attach_new_nodes(node, results, unwrapped_vec, ref_seq, ref_peptides, gene_map, virus_properties, div_units);
    }else{
      for v in unwrapped_vec{
        let pos = if let Some(pos) = results.get(*v) { pos } else { todo!() };
        let subst = pos.substitutions.iter().map(|s| s.sub.clone()).collect::<Vec<_>>();
        let dels = pos.deletions.iter().map(|s| s.del.clone()).collect::<Vec<_>>();
        let mut private_nuc_mut = find_private_nuc_mutations(
          node,
          &subst,
          &dels,
          &pos.missing,
          &Range::new(pos.alignment_start, pos.alignment_end),
          ref_seq,
          virus_properties,
        );
        let divergence_test = calculate_divergence(
          node,
          &private_nuc_mut,
          div_units, 
          ref_seq.len()
        );
        debug_assert_eq!(pos.divergence, divergence_test);
        attach_new_node(node, pos);
      }
    }
  }
}

fn attach_new_nodes(node: &mut AuspiceTreeNode, results: &[NextcladeOutputs], positions: &Vec<usize>, ref_seq: &[Nuc], ref_peptides: &TranslationMap, gene_map: &GeneMap, virus_properties: &VirusProperties, div_units: &DivergenceUnits) {
  //compute subtree
  let dist_results = calculate_distance_matrix(node, results, positions);
  let mut dist_matrix = dist_results.0;
  let mut element_order = dist_results.1;
  let mut g = build_undirected_subtree(dist_matrix, element_order);
  let parent_node = NodeType::TreeNode(TreeNode::new(node.tmp.id));
  let directed_g = build_directed_subtree(&parent_node, &g);
  //compute vertices mutations
  let mut vertices = HashMap::<NodeType, InternalMutations>::new();
  add_mutations_to_vertices(&parent_node, &directed_g, results, &mut vertices);
  //attach subtree to node
  attach_subtree(node, &parent_node, &directed_g, results, &vertices, ref_seq, ref_peptides, gene_map, virus_properties, div_units);
}

//attach subtree to node
fn attach_subtree(auspice_node: &mut AuspiceTreeNode, graph_node: &NodeType, subtree: &Graph::<NodeType, f64>, results: &[NextcladeOutputs], vertices: &HashMap::<NodeType, InternalMutations>, ref_seq: &[Nuc], ref_peptides: &TranslationMap, gene_map: &GeneMap, virus_properties: &VirusProperties, div_units: &DivergenceUnits){

  let mut pre_nodes_to_attach = subtree.adjacency.get(graph_node).unwrap();
  //check if node is a singleton
  if pre_nodes_to_attach.len()==1{
    let t_n = (pre_nodes_to_attach[0]).0;
    let nodes_to_attach_test = subtree.adjacency.get(&t_n).unwrap();
    if nodes_to_attach_test.len()>0{
      pre_nodes_to_attach = nodes_to_attach_test;
    }
  }
  let nodes_to_attach = pre_nodes_to_attach;

  for v in nodes_to_attach{
    let t_n = (*v).0;
    let vertex_result = vertices.get(&t_n).unwrap();
    if let NodeType::NewSeqNode(_) = t_n {
      let index = extract_enum_value!(t_n, NodeType::NewSeqNode(c) => c);
      let result = if let Some(pos) = results.get(index.0) { pos } else { todo!() };
      let new_mutations = recalculate_private_mutations(auspice_node, vertex_result, ref_seq, ref_peptides, gene_map, virus_properties);
      add_child(auspice_node, result, Some(new_mutations));
    }else if let NodeType::NewInternalNode(_) = t_n {
      let index = extract_enum_value!(t_n, NodeType::NewInternalNode(c) => c);
      let mut vert = compute_child(auspice_node, &index.0, vertex_result, ref_seq, ref_peptides, gene_map, virus_properties, div_units);
      attach_subtree(&mut vert, &t_n, subtree, results, vertices, ref_seq, ref_peptides, gene_map, virus_properties, div_units);
      add_computed_child(auspice_node, vert)
    }
  }
}

/// Attaches a new node to the reference tree
fn attach_new_node(node: &mut AuspiceTreeNode, result: &NextcladeOutputs) {
  debug_assert!(node.is_ref_node());
  debug_assert_eq!(node.tmp.id, result.nearest_node_id);

  if node.is_leaf() {
    add_aux_node(node);
  }

  add_child(node, result, None);
}

fn add_aux_node(node: &mut AuspiceTreeNode) {
  debug_assert!(node.is_ref_node());

  let mut aux_node = node.clone();
  aux_node.branch_attrs.mutations.clear();
  // Remove other branch attrs like labels to prevent duplication
  aux_node.branch_attrs.other = serde_json::Value::default();
  node.children.push(aux_node);

  node.name = format!("{}_parent", node.name);
}

fn add_child(node: &mut AuspiceTreeNode, result: &NextcladeOutputs, new_mutations: Option<BTreeMap<String, Vec<String>>>) {
  
  let mutations = match new_mutations {
    None => convert_mutations_to_node_branch_attrs(result),
    Some(ref x) =>  new_mutations.unwrap(),
  };

  let alignment = format!(
    "start: {}, end: {} (score: {})",
    result.alignment_start, result.alignment_end, result.alignment_score
  );

  let (has_pcr_primer_changes, pcr_primer_changes) = if result.total_pcr_primer_changes > 0 {
    (Some(TreeNodeAttr::new("No")), None)
  } else {
    (
      Some(TreeNodeAttr::new("Yes")),
      Some(TreeNodeAttr::new(&format_pcr_primer_changes(
        &result.pcr_primer_changes,
        ", ",
      ))),
    )
  };

  #[allow(clippy::from_iter_instead_of_collect)]
  let other = serde_json::Value::from_iter(
    result
      .custom_node_attributes
      .clone()
      .into_iter()
      .map(|(key, val)| (key, json!({ "value": val }))),
  );

  node.children.insert(
    0,
    AuspiceTreeNode {
      name: format!("{}_new", result.seq_name),
      branch_attrs: TreeBranchAttrs {
        mutations,
        other: serde_json::Value::default(),
      },
      node_attrs: TreeNodeAttrs {
        div: Some(result.divergence),
        clade_membership: TreeNodeAttr::new(&result.clade),
        node_type: Some(TreeNodeAttr::new("New")),
        region: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        country: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        division: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        alignment: Some(TreeNodeAttr::new(&alignment)),
        missing: Some(TreeNodeAttr::new(&format_missings(&result.missing, ", "))),
        gaps: Some(TreeNodeAttr::new(&format_nuc_deletions(&result.deletions, ", "))),
        non_acgtns: Some(TreeNodeAttr::new(&format_non_acgtns(&result.non_acgtns, ", "))),
        has_pcr_primer_changes,
        pcr_primer_changes,
        missing_genes: Some(TreeNodeAttr::new(&format_failed_genes(&result.missing_genes, ", "))),
        qc_status: Some(TreeNodeAttr::new(&result.qc.overall_status.to_string())),
        other,
      },
      children: vec![],
      tmp: TreeNodeTempData::default(),
      other: serde_json::Value::default(),
    },
  );
}

fn convert_mutations_to_node_branch_attrs(result: &NextcladeOutputs) -> BTreeMap<String, Vec<String>> {
  let NextcladeOutputs {
    private_nuc_mutations,
    private_aa_mutations,
    ..
  } = result;

  let mut mutations = BTreeMap::<String, Vec<String>>::new();

  mutations.insert(
    "nuc".to_owned(),
    convert_nuc_mutations_to_node_branch_attrs(private_nuc_mutations),
  );

  for (gene_name, aa_mutations) in private_aa_mutations {
    mutations.insert(
      gene_name.clone(),
      convert_aa_mutations_to_node_branch_attrs(aa_mutations),
    );
  }

  mutations
}

fn convert_private_mutations_to_node_branch_attrs(private_nuc_mutations: &PrivateNucMutations, private_aa_mutations: &BTreeMap<String,PrivateAaMutations>) -> BTreeMap<String, Vec<String>> {

  let mut mutations = BTreeMap::<String, Vec<String>>::new();

  mutations.insert(
    "nuc".to_owned(),
    convert_nuc_mutations_to_node_branch_attrs(private_nuc_mutations),
  );

  for (gene_name, aa_mutations) in private_aa_mutations {
    mutations.insert(
      gene_name.clone(),
      convert_aa_mutations_to_node_branch_attrs(aa_mutations),
    );
  }

  mutations
}

fn convert_nuc_mutations_to_node_branch_attrs(private_nuc_mutations: &PrivateNucMutations) -> Vec<String> {
  let dels_as_subs = private_nuc_mutations
    .private_deletions
    .iter()
    .map(NucDelMinimal::to_sub)
    .collect_vec();

  let mut subs = concat_to_vec(&private_nuc_mutations.private_substitutions, &dels_as_subs);
  subs.sort();

  subs.iter().map(NucSub::to_string).collect_vec()
}

fn convert_aa_mutations_to_node_branch_attrs(private_aa_mutations: &PrivateAaMutations) -> Vec<String> {
  let dels_as_subs = private_aa_mutations
    .private_deletions
    .iter()
    .map(AaDelMinimal::to_sub)
    .collect_vec();

  let mut subs = concat_to_vec(&private_aa_mutations.private_substitutions, &dels_as_subs);
  subs.sort();

  subs.iter().map(AaSubMinimal::to_string_without_gene).collect_vec()
}

fn recalculate_private_mutations(node: &mut AuspiceTreeNode, result: &InternalMutations, ref_seq: &[Nuc], ref_peptides: &TranslationMap, gene_map: &GeneMap, virus_properties: &VirusProperties) -> BTreeMap<String, Vec<String>>{
  let subst = result.substitutions.iter().map(|s| s.sub.clone()).collect::<Vec<_>>();
  let dels = result.deletions.iter().map(|s| s.del.clone()).collect::<Vec<_>>();
  let mut private_nuc_mut = find_private_nuc_mutations(
    node,
    &subst,
    &dels,
    &result.missing,
    &Range::new(result.alignment_start, result.alignment_end),
    ref_seq,
    virus_properties,
  );
  let aa_subst = result.aa_substitutions.iter().map(|s| s.sub.clone()).collect::<Vec<_>>();
  let aa_dels = result.aa_deletions.iter().map(|s| s.del.clone()).collect::<Vec<_>>();
  let mut private_aa_mut = find_private_aa_mutations(
    node,
    &aa_subst,
    &aa_dels,
    &result.unknown_aa_ranges,
    ref_peptides,
    gene_map,
  );
  let mutations = convert_private_mutations_to_node_branch_attrs(&private_nuc_mut, &private_aa_mut);
  
  mutations
}

fn compute_child(node: &mut AuspiceTreeNode, index: &usize, result: &InternalMutations, ref_seq: &[Nuc], ref_peptides: &TranslationMap, gene_map: &GeneMap, virus_properties: &VirusProperties, div_units: &DivergenceUnits) -> AuspiceTreeNode {

  let mutations = recalculate_private_mutations(node, result, ref_seq, ref_peptides, gene_map, virus_properties);
  let subst = result.substitutions.iter().map(|s| s.sub.clone()).collect::<Vec<_>>();
  let dels = result.deletions.iter().map(|s| s.del.clone()).collect::<Vec<_>>();
  let mut private_nuc_mut = find_private_nuc_mutations(
    node,
    &subst,
    &dels,
    &result.missing,
    &Range::new(result.alignment_start, result.alignment_end),
    ref_seq,
    virus_properties,
  );
  private_nuc_mut.total_private_substitutions = private_nuc_mut.total_private_substitutions - private_nuc_mut.total_reversion_substitutions;
  let parent_div = node.node_attrs.div.unwrap_or(0.0);
  let divergence = calculate_divergence(
    node,
    &private_nuc_mut,
    div_units, 
    ref_seq.len()
  );
  let alignment = format!(
    "start: {}, end: {} (score: {})",
    result.alignment_start, result.alignment_end, result.alignment_score
  );
 
  let mut new_node =   AuspiceTreeNode {
      name: format!("{}_new_subtree", index),
      branch_attrs: TreeBranchAttrs {
        mutations,
        other: serde_json::Value::default(),
      },
      node_attrs: TreeNodeAttrs {
        div: Some(divergence),
        clade_membership: TreeNodeAttr::new(&node.clade()),
        node_type: Some(TreeNodeAttr::new("New")),
        region: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        country: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        division: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        alignment: Some(TreeNodeAttr::new(&alignment)),
        missing: Some(TreeNodeAttr::new(&format_missings(&result.missing, ", "))),
        gaps: Some(TreeNodeAttr::new(&format_nuc_deletions(&result.deletions, ", "))),
        non_acgtns: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        has_pcr_primer_changes: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        pcr_primer_changes: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        missing_genes: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        qc_status: Some(TreeNodeAttr::new(AUSPICE_UNKNOWN_VALUE)),
        other: serde_json::Value::default(),
      },
      children: vec![],
      tmp: TreeNodeTempData::default(),
      other: serde_json::Value::default(),
    };
  let mut nuc_muts: BTreeMap<usize, Nuc> = map_nuc_muts(&new_node, ref_seq, &node.tmp.mutations).unwrap();
  let nuc_subs: BTreeMap<usize, Nuc> = nuc_muts.clone().into_iter().filter(|(_, nuc)| !nuc.is_gap()).collect();
  
  let mut aa_muts: BTreeMap<String, BTreeMap<usize, Aa>> = map_aa_muts(&new_node, ref_peptides, &node.tmp.aa_mutations).unwrap();
  let aa_subs: BTreeMap<String, BTreeMap<usize, Aa>> = aa_muts
    .clone()
    .into_iter()
    .map(|(gene, aa_muts)| (gene, aa_muts.into_iter().filter(|(_, aa)| !aa.is_gap()).collect()))
    .collect();
  
  new_node.tmp.mutations = nuc_muts.clone();
  new_node.tmp.substitutions = nuc_subs;
  new_node.tmp.aa_mutations = aa_muts.clone();
  new_node.tmp.aa_substitutions = aa_subs;
  
  new_node
}

fn add_computed_child(node: &mut AuspiceTreeNode, new_node: AuspiceTreeNode){
  node.children.insert(
    0, new_node);
}
