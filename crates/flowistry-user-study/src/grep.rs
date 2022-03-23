use std::{path::Path, process::Command};

pub fn grep(path: impl AsRef<Path>, query: &str) -> Vec<String> {
  let path = path.as_ref().canonicalize().unwrap();
  let prefix = path.parent().unwrap().display().to_string();

  let mut p = Command::new("grep");

  let mut add_arg = |s| {
    p.arg(s);
  };

  println!("Adding arguments to process");

  let should_recurse = path.is_file();
  if should_recurse {
    println!("In recurse mode");
    add_arg("-r");
  }

  add_arg(query);
  add_arg(&path.display().to_string());

  println!("Executing grep");
  let result = p.output().unwrap();
  assert!(
    result.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );

  String::from_utf8_lossy(&result.stdout)
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
    .collect::<Vec<_>>()
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
