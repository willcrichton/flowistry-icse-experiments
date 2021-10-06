use std::{
  env,
  process::{exit, Command},
};

fn main() {
  let mut cmd = Command::new("cargo");
  cmd.env("RUSTC_WORKSPACE_WRAPPER", "flowistry-eval-driver");
  cmd.args(&["+nightly-2021-09-23", "rustc", "--profile", "check"]);
  cmd.args(&env::args().skip(2).collect::<Vec<_>>());
  cmd.args(&["--", "--flowistry-eval"]);

  let exit_status = cmd.status().expect("could not run cargo");
  if !exit_status.success() {
    exit(exit_status.code().unwrap_or(-1));
  }
}
