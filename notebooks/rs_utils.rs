// serde_json = "1.0"
// serde = {version = "1.0", features = ["derive"]}
// pythonize = "0.14"
// rayon = "1.5"
// itertools = "0.10"
#![feature(rustc_private)]

extern crate rustc_macros;
extern crate rustc_serialize;

use itertools::Itertools;
use pyo3::prelude::*;
use pyo3::{exceptions::PyException, wrap_pyfunction};
use rayon::prelude::*;
use rustc_macros::Decodable;
use rustc_serialize::json;
use serde::Serialize;
use std::fs;

#[derive(Serialize, Decodable, Clone)]
pub struct Range {
  pub start: usize,
  pub end: usize,
  pub filename: String,
}

#[derive(Serialize, Decodable, Clone, PartialEq, Eq)]
pub enum MutabilityMode {
  DistinguishMut,
  IgnoreMut,
}

#[derive(Serialize, Decodable, Clone, PartialEq, Eq)]
pub enum ContextMode {
  SigOnly,
  Recurse,
}

#[derive(Serialize, Decodable, Clone, PartialEq, Eq)]
pub enum PointerMode {
  Precise,
  Conservative,
}

#[derive(Serialize, Decodable, Clone)]
pub struct EvalResult {
  mutability_mode: MutabilityMode,
  context_mode: ContextMode,
  pointer_mode: PointerMode,
  sliced_local: usize,
  function_range: Range,
  function_path: String,
  num_instructions: usize,
  num_relevant_instructions: usize,
  duration: f64,
  has_immut_ptr_in_call: bool,
  has_same_type_ptrs_in_call: bool,
  has_same_type_ptrs_in_input: bool,
  reached_library: bool,
  // added fields
  instructions_relative: Option<usize>,
  instructions_relative_frac: Option<f64>,
  instructions_relative_base: Option<usize>,
  instructions_relative_base_frac: Option<f64>,
  baseline_reached_library: Option<bool>,
}

#[pyfunction]
fn parse_data(py: Python, path: String) -> PyResult<PyObject> {
  let contents = String::from_utf8(fs::read(&path)?)?;
  let data: Vec<EvalResult> = json::decode(&contents).map_err(|err| PyException::new_err(format!("{}", err)))?;
  let mut trials = data
    .into_iter()
    .into_group_map_by(|sample| (sample.function_path.clone(), sample.sliced_local))
    .into_values()
    .collect::<Vec<_>>();

  let updated_data = trials
    .par_iter_mut()
    .map(|trial| {
      let min_sample = trial
        .iter()
        .min_by_key(|sample| sample.num_relevant_instructions)
        .cloned()
        .unwrap();
      let base_sample = trial
        .iter()
        .find(|sample| {
          sample.mutability_mode == MutabilityMode::DistinguishMut
            && sample.context_mode == ContextMode::SigOnly
            && sample.pointer_mode == PointerMode::Precise
        })
        .cloned()
        .unwrap();
      trial
        .into_iter()
        .map(|mut sample| {
          let frac = |a, b| (a - b) as f64 / (if b == 0 { 1.0 } else { b as f64 });
          let min_inst = min_sample.num_relevant_instructions;
          sample.instructions_relative = Some(sample.num_relevant_instructions - min_inst);
          sample.instructions_relative_frac =
            Some(frac(sample.num_relevant_instructions, min_inst));
          sample.reached_library = min_sample.reached_library;
          let base_inst = base_sample.num_relevant_instructions;
          sample.instructions_relative_base = Some(sample.num_relevant_instructions - base_inst);
          sample.instructions_relative_base_frac =
            Some(frac(sample.num_relevant_instructions, base_inst));
          sample.baseline_reached_library = Some(min_sample.reached_library);
          sample
        })
        .collect::<Vec<_>>()
    })
    .flatten()
    .collect::<Vec<_>>();

  Ok(pythonize::pythonize(py, &updated_data)?)
}

#[pymodule]
fn rs_utils(py: Python, m: &PyModule) -> PyResult<()> {
  m.add_function(wrap_pyfunction!(parse_data, m)?)?;
  Ok(())
}
