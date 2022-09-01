use std::{env, iter::FromIterator, time::Instant};

use flowistry::{
  infoflow::Direction,
  mir::{borrowck_facts, utils::BodyExt},
  source_map::{Range, SpanTree, ToSpan},
};
use log::info;
use rustc_ast::{
  token::Token,
  tokenstream::{TokenStream, TokenTree},
};
use rustc_data_structures::fx::FxHashSet as HashSet;
use rustc_hir::{intravisit::Visitor, BodyId, ImplItemKind, ItemKind};
use rustc_middle::{hir::nested_filter::OnlyBodies, ty::TyCtxt};
use rustc_span::{source_map::Spanned, FileName, Span, SpanData, SyntaxContext};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EvalResult {
  function_range: Range,
  function_path: String,
  range: Range,
  num_instructions: usize,
  num_tokens: usize,
  num_lines: usize,
  direction: Direction,
  num_relevant_tokens: usize,
  num_relevant_lines: usize,
  line_iqr: usize,
  duration: f64,
}

struct Tokens {
  spans: SpanTree<usize>,
}

impl Tokens {
  fn flatten_stream(stream: TokenStream) -> Vec<Token> {
    stream
      .into_trees()
      .flat_map(|tree| match tree {
        TokenTree::Token(token) => vec![token],
        TokenTree::Delimited(_, _, stream) => Self::flatten_stream(stream),
      })
      .collect()
  }

  pub fn build(tcx: TyCtxt<'_>, span: Span, count: usize) -> Self {
    log::debug!("Tokens: {span:?}");
    let source_map = tcx.sess.source_map();
    let snippet = source_map.span_to_snippet(span).unwrap();
    log::debug!("{snippet}");

    let base = span.lo();
    let mut parser = rustc_parse::new_parser_from_source_str(
      &tcx.sess.parse_sess,
      FileName::Anon(count as u64),
      snippet,
    );

    let token_stream = parser.parse_tokens();
    let tokens = Self::flatten_stream(token_stream);
    log::debug!(
      "{:?}",
      tokens.iter().map(|token| &token.kind).collect::<Vec<_>>()
    );

    let spans = SpanTree::new(tokens.into_iter().enumerate().map(|(idx, token)| {
      let lo = source_map.lookup_byte_offset(token.span.lo()).pos;
      let hi = source_map.lookup_byte_offset(token.span.hi()).pos;
      let span = Span::new(base + lo, base + hi, SyntaxContext::root(), None);
      log::debug!("{span:?}");
      Spanned { span, node: idx }
    }));
    Tokens { spans }
  }

  pub fn total_tokens(&self) -> usize {
    self.spans.len()
  }

  pub fn query(
    &self,
    spans: impl IntoIterator<Item = Span>,
  ) -> HashSet<&(SpanData, usize)> {
    spans
      .into_iter()
      .flat_map(|span| self.spans.overlapping(span.data()))
      .collect::<HashSet<_>>()
  }
}

pub trait BodyVisitor<'tcx> {
  fn visit(&mut self, body_span: Span, body_id: BodyId, tcx: TyCtxt<'tcx>);
}

struct BodyFinder<'tcx, 'a, V> {
  pub tcx: TyCtxt<'tcx>,
  pub visitor: &'a mut V,
}

impl<'tcx, V> Visitor<'tcx> for BodyFinder<'tcx, '_, V>
where
  V: BodyVisitor<'tcx>,
{
  type NestedFilter = OnlyBodies;

  fn nested_visit_map(&mut self) -> Self::Map {
    self.tcx.hir()
  }

  fn visit_nested_body(&mut self, id: BodyId) {
    let hir = self.nested_visit_map();

    // const/static items are considered to have bodies, so we want to exclude
    // them from our search for functions
    if !hir
      .body_owner_kind(hir.body_owner_def_id(id))
      .is_fn_or_closure()
    {
      return;
    }

    let owner = hir.body_owner(id);
    let body_span = hir.span(owner);
    if body_span.from_expansion() {
      return;
    }

    self.visitor.visit(body_span, id, self.tcx);
  }
}

pub fn visit_bodies<'tcx, V: BodyVisitor<'tcx>>(tcx: TyCtxt<'tcx>, visitor: &mut V) {
  tcx
    .hir()
    .deep_visit_all_item_likes(&mut BodyFinder { tcx, visitor });
}

pub struct EvalCrateVisitor {
  count: usize,
  total: usize,
  pub eval_results: Vec<EvalResult>,
}

impl BodyVisitor<'_> for EvalCrateVisitor {
  fn visit(&mut self, body_span: Span, body_id: BodyId, tcx: TyCtxt) {
    let source_map = tcx.sess.source_map();
    let source_file = &source_map.lookup_source_file(body_span.lo());
    if source_file.src.is_none() {
      return;
    }

    let function_range = &match Range::from_span(body_span, source_map) {
      Ok(range) => range,
      Err(_) => {
        return;
      }
    };

    self.count += 1;

    let local_def_id = tcx.hir().body_owner_def_id(body_id);
    let def_id = local_def_id.to_def_id();
    let function_path = &tcx.def_path_debug_str(def_id);

    let only_run = env::var("ONLY_RUN");
    if let Ok(n) = only_run {
      let skip = match n.parse::<usize>() {
        Ok(n) => self.count != n,
        Err(_) => function_path != &n,
      };
      if skip {
        return;
      }
    }

    info!(
      "Visiting {} ({} / {})",
      function_path, self.count, self.total
    );

    let start = Instant::now();
    let body_with_facts = borrowck_facts::get_body_with_borrowck_facts(tcx, local_def_id);
    let facts_duration = start.elapsed().as_secs_f64();
    let body = &body_with_facts.body;
    let num_instructions = body.all_locations().count();

    let body_span = tcx.hir().body(body_id).value.span;
    let start = Instant::now();
    let tokens = Tokens::build(tcx, body_span, self.count);
    let build_duration = start.elapsed().as_secs_f64();
    let num_tokens = tokens.total_tokens();

    let span_lines = |sp: Span| {
      let lines = source_map.span_to_lines(sp).unwrap().lines;
      lines.first().unwrap().line_index ..= lines.last().unwrap().line_index
    };

    let mut body_lines = tokens
      .spans
      .spans()
      .flat_map(|span| span_lines(span.span()))
      .collect::<Vec<_>>();
    body_lines.dedup();
    body_lines.sort();
    let num_lines = body_lines.len();

    let start = Instant::now();
    fluid_let::fluid_set!(flowistry_ide::FOCUS_DEBUG, true);
    let focus = flowistry_ide::focus(tcx, body_id).unwrap();
    let duration = start.elapsed().as_secs_f64();

    let start = Instant::now();
    let eval_results = focus.place_info.into_iter().flat_map(|place_info| {
      [Direction::Forward, Direction::Backward, Direction::Both]
        .into_iter()
        .map(|direction| {
          let slice = match direction {
            Direction::Both => place_info.slice.iter(),
            Direction::Forward => place_info.forward.as_ref().unwrap().iter(),
            Direction::Backward => place_info.backward.as_ref().unwrap().iter(),
          };
          let spans = slice.map(|range| range.to_span(tcx).unwrap());

          let mut relevant_tokens = Vec::from_iter(tokens.query(spans));
          relevant_tokens.sort_by_key(|(_, idx)| *idx);
          let num_relevant_tokens = relevant_tokens.len();

          let mut relevant_lines = relevant_tokens
            .iter()
            .flat_map(|(span, _)| span_lines(span.span()))
            .collect::<Vec<_>>();
          relevant_lines.dedup();
          relevant_lines.sort();

          let num_relevant_lines = relevant_lines.len();

          let n = num_relevant_lines;
          let line_iqr = if n > 0 {
            let lo = relevant_lines[n * 1 / 4];
            let hi = relevant_lines[n * 3 / 4];
            body_lines.iter().filter(|i| lo <= **i && **i <= hi).count()
          } else {
            0
          };

          EvalResult {
            // function-level data
            function_range: function_range.clone(),
            function_path: function_path.clone(),
            num_instructions,
            num_tokens,
            num_lines,
            //
            // sample-level parameters
            range: place_info.range.clone(),
            direction,
            //
            // sample-level data
            num_relevant_tokens,
            num_relevant_lines,
            line_iqr,
            duration,
          }
        })
        .collect::<Vec<_>>()
    });

    self.eval_results.extend(eval_results);
    let output_duration = start.elapsed().as_secs_f64();
    info!("facts={facts_duration:.3} build={build_duration:.3} analyze={duration:.3} output={output_duration:.3}");
  }
}

impl EvalCrateVisitor {
  pub fn new(total: usize) -> Self {
    EvalCrateVisitor {
      count: 0,
      total,
      eval_results: Vec::new(),
    }
  }
}

pub struct ItemCounter {
  pub count: usize,
}

impl BodyVisitor<'_> for ItemCounter {
  fn visit(&mut self, body_span: Span, body_id: BodyId, tcx: TyCtxt) {
    let source_map = tcx.sess.source_map();
    let source_file = &source_map.lookup_source_file(body_span.lo());
    if source_file.src.is_none() {
      return;
    }

    self.count += 1;
  }
}
