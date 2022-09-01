#![feature(rustc_private, box_patterns)]

extern crate either;
extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_macros;
extern crate rustc_middle;
extern crate rustc_parse;
extern crate rustc_serialize;
extern crate rustc_span;

mod visitor;

use std::{env, fs};

use flowistry::mir::borrowck_facts;

struct Callbacks {
  output_path: String,
}

impl rustc_driver::Callbacks for Callbacks {
  fn config(&mut self, config: &mut rustc_interface::Config) {
    // You MUST configure rustc to ensure `get_body_with_borrowck_facts` will work.
    config.override_queries = Some(borrowck_facts::override_queries);
  }

  fn after_parsing<'tcx>(
    &mut self,
    _compiler: &rustc_interface::interface::Compiler,
    queries: &'tcx rustc_interface::Queries<'tcx>,
  ) -> rustc_driver::Compilation {
    queries.global_ctxt().unwrap().take().enter(|tcx| {
      let mut counter = visitor::ItemCounter { count: 0 };
      visitor::visit_bodies(tcx, &mut counter);

      let mut eval_visitor = visitor::EvalCrateVisitor::new(counter.count);
      visitor::visit_bodies(tcx, &mut eval_visitor);

      let json = serde_json::to_string(&eval_visitor.eval_results).unwrap();

      fs::write(&self.output_path, &json).unwrap();
    });

    rustc_driver::Compilation::Stop
  }
}

pub fn run(args: &[String]) -> rustc_interface::interface::Result<()> {
  let mut callbacks = Callbacks {
    output_path: env::var("OUTPUT_PATH").unwrap(),
  };
  rustc_driver::RunCompiler::new(args, &mut callbacks).run()
}
