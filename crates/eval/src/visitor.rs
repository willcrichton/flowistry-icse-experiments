use std::{env, time::Instant};

use either::Either;
use flowistry::{
  infoflow::Direction,
  mir::{borrowck_facts, utils::BodyExt},
  range::Range,
};
use log::info;
use rustc_hir::{itemlikevisit::ItemLikeVisitor, BodyId, ImplItemKind, ItemKind};
use rustc_macros::Encodable;
use rustc_middle::{
  mir::{Statement, StatementKind, Terminator, TerminatorKind},
  ty::TyCtxt,
};
use rustc_span::Span;

pub struct EvalCrateVisitor<'tcx> {
  tcx: TyCtxt<'tcx>,
  count: usize,
  total: usize,
  pub eval_results: Vec<EvalResult>,
}

#[derive(Debug, Encodable)]
pub struct EvalResult {
  location: String,
  direction: String,
  function_range: Range,
  function_path: String,
  num_instructions: usize,
  num_relevant_instructions: usize,
  duration: f64,
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

    let function_range = &match Range::from_span(body_span, source_map) {
      Ok(range) => range,
      Err(_) => {
        return;
      }
    };

    let local_def_id = tcx.hir().body_owner_def_id(*body_id);
    let def_id = local_def_id.to_def_id();
    let function_path = &tcx.def_path_debug_str(def_id);
    info!("Visiting {} ({} / {})", function_path, count, self.total);

    let body_with_facts = borrowck_facts::get_body_with_borrowck_facts(tcx, local_def_id);
    let body = &body_with_facts.body;

    let num_instructions = body.all_locations().count();

    let start = Instant::now();
    let results = flowistry::infoflow::compute_flow(tcx, *body_id, &body_with_facts);
    let duration = start.elapsed().as_secs_f64();

    let targets = body
      .all_locations()
      .filter_map(|location| match body.stmt_at(location) {
        Either::Left(Statement {
          kind: StatementKind::Assign(box (lhs, _)),
          ..
        })
        | Either::Right(Terminator {
          kind:
            TerminatorKind::Call {
              destination: Some((lhs, _)),
              ..
            },
          ..
        }) => Some(vec![(*lhs, location)]),
        _ => None,
      })
      .collect::<Vec<_>>();

    let eval_results = [Direction::Forward, Direction::Backward, Direction::Both]
      .iter()
      .flat_map(|direction| {
        let deps = flowistry::infoflow::compute_dependencies(
          &results,
          targets.clone(),
          *direction,
        );

        targets
          .iter()
          .zip(deps.into_iter())
          .map(|(target, deps)| {
            EvalResult {
              // function-level data
              function_range: function_range.clone(),
              function_path: function_path.clone(),
              num_instructions,
              //
              // sample-level parameters
              location: format!("{:?}", target[0].1),
              direction: (match direction {
                Direction::Forward => "forward",
                Direction::Backward => "backward",
                Direction::Both => "both",
              })
              .into(),
              //
              // sample-level data
              num_relevant_instructions: deps
                .iter()
                .filter(|location| {
                  results
                    .analysis
                    .location_domain()
                    .location_to_local(**location)
                    .is_none()
                })
                .count(),
              duration,
            }
          })
          .collect::<Vec<_>>()
      });

    self.eval_results.extend(eval_results);
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
