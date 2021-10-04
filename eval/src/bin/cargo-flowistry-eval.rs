use std::env;
use std::process::{exit, Command};

fn main() {
  let mut cmd = Command::new("cargo");
  cmd.env("RUSTC_WORKSPACE_WRAPPER", "flowistry-eval-driver");
  cmd.args(&["rustc", "--profile", "check", "-q"]);
  cmd.args(&env::args().skip(1).collect::<Vec<_>>());
  cmd.args(&["--", "--flowistry-eval"]);

  let exit_status = cmd.status().expect("could not run cargo");
  if !exit_status.success() {
    exit(exit_status.code().unwrap_or(-1));
  }
}
