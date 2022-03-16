use std::collections::HashMap;

pub struct PositionalArg {
  name: String,
}

pub struct NamedArg {
  name: String,
  required: bool,
  takes_value: bool,
}

#[derive(Default)]
pub struct Cli {
  positional: Vec<PositionalArg>,
  named: HashMap<String, NamedArg>,
}

impl Cli {
  pub fn arg(&mut self, name: String) -> &mut Self {
    self.positional.push(PositionalArg { name });
    self
  }

  pub fn flag(&mut self, name: String, required: bool, takes_value: bool) -> &mut Self {
    self.named.insert(
      name.clone(),
      NamedArg {
        name,
        required,
        takes_value,
      },
    );
    self
  }

  pub fn parse(&mut self, args: &[String]) -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    let mut i = 0;
    let positional = &mut self.positional.iter();

    while let Some(arg) = args.get(i) {
      if arg.starts_with("--") {
        let flag = &arg[2..];
        let entry = self.named.get(flag)?;
        let v = if entry.takes_value {
          i += 1;
          s(args.get(i)?)
        } else {
          s("")
        };
        map.insert(s(flag), v);
      } else {
        let entry = positional.next()?;
        map.insert(entry.name.clone(), arg.clone());
      }

      i += 1;
    }

    Some(map)
  }
}

fn s(s: &str) -> String {
  s.to_string()
}

#[test]
fn cli_test1() {
  use maplit::hashmap;

  let mut cli = Cli::default();
  cli
    .flag(s("verbose"), false, false)
    .flag(s("output-type"), false, true)
    .arg(s("file"));
  assert_eq!(
    Some(hashmap! {
      s("verbose") => s(""),
      s("output-type") => s("json"),
      s("file") => s("input.txt")
    }),
    cli.parse(&[
      s("--verbose"),
      s("--output-type"),
      s("json"),
      s("input.txt")
    ])
  );
}
