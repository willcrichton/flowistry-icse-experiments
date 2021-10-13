use fluid_let::fluid_set;
use itertools::iproduct;
use log::info;
use rustc_data_structures::fx::FxHashMap as HashMap;
use rustc_hir::{itemlikevisit::ItemLikeVisitor, BodyId, ImplItemKind, ItemKind};
use rustc_macros::Encodable;
use rustc_middle::{
  mir::{
    visit::Visitor, Body, HasLocalDecls, Location, Mutability, Place, Terminator, TerminatorKind,
  },
  ty::{Ty, TyCtxt, TyS},
};
use rustc_span::{def_id::DefId, Span};
use std::{cell::RefCell, env, time::Instant};

use flowistry::{
  extensions::{ContextMode, EvalMode, MutabilityMode, PointerMode, EVAL_MODE, REACHED_LIBRARY},
  utils, Direction, Range,
};

struct EvalBodyVisitor<'a, 'tcx> {
  tcx: TyCtxt<'tcx>,
  body: &'a Body<'tcx>,
  def_id: DefId,
  has_immut_ptr_in_call: bool,
  has_same_type_ptrs_in_call: bool,
  has_same_type_ptrs_in_input: bool,
}

impl EvalBodyVisitor<'_, 'tcx> {
  fn place_ty(&self, place: Place<'tcx>) -> Ty<'tcx> {
    self
      .tcx
      .erase_regions(place.ty(self.body.local_decls(), self.tcx).ty)
  }

  fn any_same_type_ptrs(&self, places: Vec<Place<'tcx>>) -> bool {
    places.iter().enumerate().any(|(i, place)| {
      places
        .iter()
        .enumerate()
        .filter(|(j, _)| i != *j)
        .any(|(_, place2)| TyS::same_type(self.place_ty(*place), self.place_ty(*place2)))
    })
  }
}

impl Visitor<'tcx> for EvalBodyVisitor<'_, 'tcx> {
  fn visit_body(&mut self, body: &Body<'tcx>) {
    self.super_body(body);

    let input_ptrs = body
      .args_iter()
      .map(|local| {
        let place = utils::local_to_place(local, self.tcx);
        utils::interior_pointers(place, self.tcx, self.body, self.def_id)
          .into_values()
          .map(|v| v.into_iter())
          .flatten()
      })
      .flatten()
      .filter_map(|(place, mutability)| (mutability == Mutability::Mut).then(|| place))
      .collect::<Vec<_>>();

    let has_same_type_ptrs = self.any_same_type_ptrs(input_ptrs);
    self.has_same_type_ptrs_in_input |= has_same_type_ptrs;
  }

  fn visit_terminator(&mut self, terminator: &Terminator<'tcx>, _location: Location) {
    if let TerminatorKind::Call {
      args, destination, ..
    } = &terminator.kind
    {
      let input_ptrs = args
        .iter()
        .filter_map(|operand| utils::operand_to_place(operand))
        .map(|place| {
          utils::interior_pointers(place, self.tcx, self.body, self.def_id)
            .into_values()
            .map(|v| v.into_iter())
            .flatten()
        })
        .flatten()
        .collect::<Vec<_>>();

      let output_ptrs = destination
        .map(|(place, _)| {
          utils::interior_pointers(place, self.tcx, self.body, self.def_id)
            .into_values()
            .map(|v| v.into_iter())
            .flatten()
            .collect::<Vec<_>>()
        })
        .unwrap_or_else(Vec::new);

      let all_ptr_places = input_ptrs
        .clone()
        .into_iter()
        .chain(output_ptrs.into_iter())
        .filter_map(|(place, mutability)| (mutability == Mutability::Mut).then(|| place))
        .collect::<Vec<_>>();

      let has_immut_ptr = input_ptrs
        .iter()
        .any(|(_, mutability)| *mutability == Mutability::Not);

      let has_same_type_ptrs = self.any_same_type_ptrs(all_ptr_places);

      self.has_immut_ptr_in_call |= has_immut_ptr;
      self.has_same_type_ptrs_in_call |= has_same_type_ptrs;
    }
  }
}

pub struct EvalCrateVisitor<'tcx> {
  tcx: TyCtxt<'tcx>,
  count: usize,
  total: usize,
  pub eval_results: Vec<EvalResult>,
}

#[derive(Debug, Encodable)]
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
}

impl EvalCrateVisitor<'tcx> {
  pub fn new(tcx: TyCtxt<'tcx>, total: usize) -> Self {
    EvalCrateVisitor {
      tcx,
      count: 0,
      total,
      eval_results: Vec::new(),
    }
  }

  fn analyze(&mut self, body_span: Span, body_id: &BodyId) {
    if body_span.from_expansion() {
      return;
    }

    let tcx = self.tcx;
    let source_map = tcx.sess.source_map();
    let source_file = &source_map.lookup_source_file(body_span.lo());
    if source_file.src.is_none() {
      return;
    }

    let count = {
      self.count += 1;
      self.count
    };

    let only_run = env::var("ONLY_RUN");
    if let Ok(n) = only_run {
      if count < n.parse::<usize>().unwrap() {
        return;
      }
    }

    let local_def_id = tcx.hir().body_owner_def_id(*body_id);
    let def_id = local_def_id.to_def_id();
    let function_path = &tcx.def_path_debug_str(def_id);
    info!("Visiting {} ({} / {})", function_path, count, self.total);

    let body_with_facts = flowistry::get_body_with_borrowck_facts(tcx, local_def_id);
    let body = &body_with_facts.body;

    let mut body_visitor = EvalBodyVisitor {
      tcx: tcx,
      body,
      def_id,
      has_immut_ptr_in_call: false,
      has_same_type_ptrs_in_call: false,
      has_same_type_ptrs_in_input: false,
    };
    body_visitor.visit_body(body);

    let exits = body
      .basic_blocks()
      .iter_enumerated()
      .filter_map(|(bb, data)| {
        matches!(data.terminator().kind, TerminatorKind::Return).then(|| body.terminator_loc(bb))
      })
      .collect::<Vec<_>>();

    let targets = body
      .local_decls
      .indices()
      .map(|local| {
        exits
          .iter()
          .map(move |exit| (utils::local_to_place(local, tcx), *exit))
      })
      .flatten()
      .collect::<Vec<_>>();

    let has_immut_ptr_in_call = body_visitor.has_immut_ptr_in_call;
    let has_same_type_ptrs_in_input = body_visitor.has_same_type_ptrs_in_input;
    let has_same_type_ptrs_in_call = body_visitor.has_same_type_ptrs_in_call;

    let function_range = &match Range::from_span(body_span, source_map) {
      Ok(range) => range,
      Err(_) => {
        return;
      }
    };

    let num_instructions = body
      .basic_blocks()
      .iter()
      .map(|data| data.statements.len() + 1)
      .sum::<usize>();

    let eval_results = iproduct!(
      vec![MutabilityMode::DistinguishMut, MutabilityMode::IgnoreMut].into_iter(),
      vec![ContextMode::Recurse, ContextMode::SigOnly].into_iter(),
      vec![PointerMode::Precise, PointerMode::Conservative].into_iter()
    )
    .map(|(mutability_mode, context_mode, pointer_mode)| {
      let eval_mode = EvalMode {
        mutability_mode,
        context_mode,
        pointer_mode,
      };
      fluid_set!(EVAL_MODE, &eval_mode);

      let reached_library = RefCell::new(false);
      fluid_set!(REACHED_LIBRARY, &reached_library);

      let start = Instant::now();
      let flow = &flowistry::compute_flow(tcx, *body_id, &body_with_facts);

      let deps = flowistry::compute_dependencies(flow, targets.clone(), Direction::Backward);
      let mut joined_deps = HashMap::default();
      for ((place, _), (locations, _)) in targets.iter().zip(deps.into_iter()) {
        joined_deps
          .entry(place.local.as_usize())
          .or_insert_with(|| locations.clone())
          .union(&locations);
      }

      let duration = (start.elapsed().as_nanos() as f64) / 10e9;

      joined_deps.into_iter().map(move |(sliced_local, deps)| {
        EvalResult {
          // function-level data
          function_range: function_range.clone(),
          function_path: function_path.clone(),
          num_instructions,
          has_immut_ptr_in_call,
          has_same_type_ptrs_in_call,
          has_same_type_ptrs_in_input,
          //
          // sample-level parameters
          context_mode,
          mutability_mode,
          pointer_mode,
          sliced_local,
          //
          // sample-level data
          num_relevant_instructions: deps.len(),
          duration,
          reached_library: *reached_library.borrow(),
        }
      })
    })
    .flatten()
    .collect::<Vec<_>>();

    self
      .eval_results
      .extend(eval_results);
  }
}

impl ItemLikeVisitor<'tcx> for EvalCrateVisitor<'tcx> {
  fn visit_item(&mut self, item: &'tcx rustc_hir::Item<'tcx>) {
    match &item.kind {
      ItemKind::Fn(_, _, body_id) => {
        self.analyze(item.span, body_id);
      }
      _ => {}
    }
  }

  fn visit_impl_item(&mut self, impl_item: &'tcx rustc_hir::ImplItem<'tcx>) {
    match &impl_item.kind {
      ImplItemKind::Fn(_, body_id) => {
        self.analyze(impl_item.span, body_id);
      }
      _ => {}
    }
  }

  fn visit_trait_item(&mut self, _trait_item: &'tcx rustc_hir::TraitItem<'tcx>) {}

  fn visit_foreign_item(&mut self, _foreign_item: &'tcx rustc_hir::ForeignItem<'tcx>) {}
}

pub struct ItemCounter<'tcx> {
  pub tcx: TyCtxt<'tcx>,
  pub count: usize,
}

impl ItemCounter<'_> {
  fn analyze(&mut self, body_span: Span) {
    if body_span.from_expansion() {
      return;
    }

    let source_map = self.tcx.sess.source_map();
    let source_file = &source_map.lookup_source_file(body_span.lo());
    if source_file.src.is_none() {
      return;
    }

    self.count += 1;
  }
}

impl ItemLikeVisitor<'tcx> for ItemCounter<'tcx> {
  fn visit_item(&mut self, item: &'tcx rustc_hir::Item<'tcx>) {
    match &item.kind {
      ItemKind::Fn(_, _, _) => {
        self.analyze(item.span);
      }
      _ => {}
    }
  }

  fn visit_impl_item(&mut self, impl_item: &'tcx rustc_hir::ImplItem<'tcx>) {
    match &impl_item.kind {
      ImplItemKind::Fn(_, _) => {
        self.analyze(impl_item.span);
      }
      _ => {}
    }
  }

  fn visit_trait_item(&mut self, _trait_item: &'tcx rustc_hir::TraitItem<'tcx>) {}

  fn visit_foreign_item(&mut self, _foreign_item: &'tcx rustc_hir::ForeignItem<'tcx>) {}
}
