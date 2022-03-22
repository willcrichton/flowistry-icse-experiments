//! This file explains how Flowistry works. Please read the comments from top-to-bottom, and
//! use Flowistry as directed.

use std::{path::Path, process::Command};

// Click on each argument to see the influence of that argument on the function.
// Notice that the focused object is dark gray, and direct references or mutations to that
// object are light gray.
//
// Then click on the return type to see what influences the return values of this function.
pub fn grep(path: impl AsRef<Path>, query: &str) -> Vec<String> {
  let path = path.as_ref().canonicalize().unwrap();

  // Click on `prefix`. Notice that this is not relevant until the `return` statement
  // below, so you could come back to this variable later.
  let prefix = path.parent().unwrap().display().to_string();

  // Click on `p`. Notice that "let path" and "let prefix" is grayed out, meaning you don't
  // need to look back to those lines to understand this one.
  let mut p = Command::new("grep");

  let mut add_arg = |s| {
    p.arg(s);
  };

  // Click on `println!`. Notice that this println doesn't affect the value of anything,
  // so it is irrelevant to everything.
  println!("Adding arguments to process");

  let should_recurse = path.is_file();
  if should_recurse {
    println!("In recurse mode");

    add_arg("-r");
  }

  add_arg(query);
  add_arg(&path.display().to_string());

  // If you want to debug why `p` is misconfigured, you could focus on `p` here.
  // Only the code relevant to `p` will show above. This includes subtle cases
  // like the fact that `add_arg` is a closure with a mutable reference to `p`.
  // Additionally, `add_arg("-r")` only executes if `should_recurse` is true, so
  // `should_recurse` is highlighted as well.
  //
  // Also, notice that clicking on `p` here is different than clicking `p` on line 14.
  // In one case, you see the *forward* influence *of* p, and in the other case you
  // see the *backward* influence *on* p.
  println!("Executing grep");
  let result = p.output().unwrap();
  assert!(
    result.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );

  return String::from_utf8_lossy(&result.stdout)
    .trim()
    .split("\n")
    .map(|s| {
      (if s.starts_with(&prefix) {
        s.split_once(':').unwrap().1
      } else {
        s
      })
      .to_owned()
    })
    .collect::<Vec<_>>();
}

#[test]
fn grep_test1() {
  assert_eq!(
    vec![r#"name = "user-study""#],
    grep("./Cargo.toml", r#""user-study""#)
  );
}

#[test]
fn grep_test2() {
  assert_eq!(
    vec!["  println!(\"Executing grep\");"],
    grep("./src", "\"Executing grep\"")
  );
}
