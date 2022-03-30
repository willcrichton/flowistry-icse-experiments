use std::{env, time::Instant};

use flowistry::{
  mir::{borrowck_facts, utils::BodyExt},
  source_map::{Range, SpanTree, ToSpan},
};
use log::info;
use rustc_ast::{
  token::Token,
  tokenstream::{TokenStream, TokenTree},
};
use rustc_data_structures::fx::FxHashSet as HashSet;
use rustc_hir::{itemlikevisit::ItemLikeVisitor, BodyId, ImplItemKind, ItemKind};
use rustc_macros::Encodable;
use rustc_middle::ty::TyCtxt;
use rustc_span::{source_map::Spanned, FileName, Span, SyntaxContext};

pub struct EvalCrateVisitor<'tcx> {
  tcx: TyCtxt<'tcx>,
  count: usize,
  total: usize,
  pub eval_results: Vec<EvalResult>,
}

#[derive(Debug, Encodable)]
pub struct EvalResult {
  function_range: Range,
  function_path: String,
  range: Range,
  num_instructions: usize,
  num_tokens: usize,
  num_relevant_tokens: usize,
  duration: f64,
}

struct Tokens {
  spans: SpanTree<()>,
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

    let spans = SpanTree::new(tokens.into_iter().map(|token| {
      let lo = source_map.lookup_byte_offset(token.span.lo()).pos;
      let hi = source_map.lookup_byte_offset(token.span.hi()).pos;
      let span = Span::new(base + lo, base + hi, SyntaxContext::root(), None);
      log::debug!("{span:?}");
      Spanned { span, node: () }
    }));
    Tokens { spans }
  }

  pub fn total_tokens(&self) -> usize {
    self.spans.len()
  }

  pub fn count_tokens_overlapping(&self, spans: impl IntoIterator<Item = Span>) -> usize {
    let all_tokens = spans
      .into_iter()
      .flat_map(|span| self.spans.overlapping(span.data()))
      .collect::<HashSet<_>>();
    all_tokens.len()
  }
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

    let function_range = &match Range::from_span(body_span, source_map) {
      Ok(range) => range,
      Err(_) => {
        return;
      }
    };

    self.count += 1;

    let local_def_id = tcx.hir().body_owner_def_id(*body_id);
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

    let body_span = tcx.hir().body(*body_id).value.span;
    let start = Instant::now();
    let tokens = Tokens::build(tcx, body_span, self.count);
    let build_duration = start.elapsed().as_secs_f64();
    let num_tokens = tokens.total_tokens();

    let start = Instant::now();
    let focus = flowistry_ide::focus::focus(tcx, *body_id).unwrap();
    let duration = start.elapsed().as_secs_f64();

    let start = Instant::now();
    let eval_results = focus.place_info.into_iter().map(|place_info| {
      let spans = place_info
        .slice
        .into_iter()
        .map(|range| range.to_span(tcx).unwrap());
      let num_relevant_tokens = tokens.count_tokens_overlapping(spans);
      EvalResult {
        // function-level data
        function_range: function_range.clone(),
        function_path: function_path.clone(),
        num_instructions,
        num_tokens,
        //
        // sample-level parameters
        range: place_info.range,
        //
        // sample-level data
        num_relevant_tokens,
        duration,
      }
    });

    self.eval_results.extend(eval_results);
    let output_duration = start.elapsed().as_secs_f64();
    info!("facts={facts_duration:.3} build={build_duration:.3} analyze={duration:.3} output={output_duration:.3}");
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
