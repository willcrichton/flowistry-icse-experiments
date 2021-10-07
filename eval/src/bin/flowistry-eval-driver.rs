#![feature(rustc_private)]

extern crate rustc_driver;

use std::{env, process::exit};

struct DefaultCallbacks;
impl rustc_driver::Callbacks for DefaultCallbacks {}

fn main() {
  rustc_driver::init_rustc_env_logger();
  env_logger::init();

  exit(rustc_driver::catch_with_exit_code(move || {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    args.extend_from_slice(&["--sysroot".into(), env::var("SYSROOT").unwrap()]);

    let mut is_driver = false;
    args.retain(|arg| {
      if arg.starts_with("--flowistry-eval") {
        is_driver = true;
        false
      } else {
        true
      }
    });

    if is_driver {
      args.push(format!(
        "-Zthreads={}",
        env::var("THREADS").unwrap_or("1".to_string())
      ));
      flowistry_eval::run(&args)
    } else {
      rustc_driver::RunCompiler::new(&args, &mut DefaultCallbacks).run()
    }
  }));
}
