//! An API for building a command-line interface (CLI). Allows users
//! to register a set of positional and named arguments, and parse those
//! arguments from a list of strings.

use std::{
  collections::{HashMap, VecDeque},
  time::Instant,
};

#[derive(Debug, Clone)]
struct PositionalArg {
  name: String,
}

#[derive(Debug, Clone)]
struct NamedArg {
  required: bool,
  takes_value: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Cli {
  positional: VecDeque<PositionalArg>,
  named: HashMap<String, NamedArg>,
}

impl Cli {
  /// Create a new CLI builder.
  pub fn new() -> Self {
    Cli {
      positional: VecDeque::new(),
      named: HashMap::new(),
    }
  }

  /// Add a named argument to the builder.
  pub fn named(mut self, name: impl Into<String>, required: bool, takes_value: bool) -> Self {
    self.named.insert(
      name.into(),
      NamedArg {
        required,
        takes_value,
      },
    );
    self
  }

  /// Add a positional argument to the builder.
  pub fn positional(mut self, name: impl Into<String>) -> Self {
    self
      .positional
      .push_back(PositionalArg { name: name.into() });
    self
  }

  /// Consume the builder and attempt to parse the input strings into a set of key -> value pairings,
  /// returning an error for inputs that don't match the builder configuration.
  pub fn parse(&self, args: Vec<String>) -> Result<HashMap<String, String>, String> {
    let mut cli = self.clone();
    let mut args = VecDeque::from(args);
    let mut parsed = HashMap::new();
    let build_error = |err: ErrorType| err.to_string(self);
    let start = Instant::now();

    args
      .pop_front()
      .ok_or_else(|| build_error(ErrorType::MissingBinary))?;

    while !args.is_empty() {
      let arg = args.pop_front().unwrap();
      match arg.split_once("--") {
        Some((_, flag)) => {
          let named_arg = cli
            .named
            .remove(flag)
            .ok_or_else(|| build_error(ErrorType::InvalidNamed(flag)))?;

          let value = if named_arg.takes_value {
            args
              .pop_front()
              .ok_or_else(|| build_error(ErrorType::MissingValue(flag)))?
          } else {
            "".to_string()
          };

          parsed.insert(flag.to_string(), value);
        }
        None => {
          let pos_arg = cli
            .positional
            .pop_front()
            .ok_or_else(|| build_error(ErrorType::InvalidPositional(&arg)))?;

          parsed.insert(pos_arg.name, arg);
        }
      }
    }

    if !cli.positional.is_empty() {
      return Err(build_error(ErrorType::MissingPositional));
    }

    if cli.named.values().any(|arg| arg.required) {
      return Err(build_error(ErrorType::MissingNamed));
    }

    log::info!("Executed parse in {}s.", start.elapsed().as_secs());

    Ok(parsed)
  }
}

#[derive(Debug)]
enum ErrorType<'a> {
  MissingBinary,
  InvalidNamed(&'a str),
  InvalidPositional(&'a str),
  MissingNamed,
  MissingPositional,
  MissingValue(&'a str),
}

impl<'a> ErrorType<'a> {
  /// Build a user-interpretable error for a given cause, and include context
  /// as appropriate.
  fn to_string(&self, cli: &Cli) -> String {
    let cause = match self {
      ErrorType::MissingBinary => "Missing binary".to_string(),

      ErrorType::InvalidNamed(name) => format!("Invalid named argument \"{name}\""),

      ErrorType::InvalidPositional(arg) => format!("Invalid positional argument \"{arg}\""),

      ErrorType::MissingPositional | ErrorType::MissingNamed => {
        let mut buf = String::new();

        let (kind, args) = match self {
          ErrorType::MissingPositional => (
            "positional",
            cli
              .positional
              .iter()
              .map(|arg| &arg.name)
              .collect::<Vec<_>>(),
          ),
          ErrorType::MissingNamed => ("named", cli.named.keys().collect::<Vec<_>>()),
          _ => unreachable!(),
        };

        buf.push_str(&format!("Missing {kind} arguments: ",));

        for (i, name) in args.iter().enumerate() {
          buf.push_str(name);
          if i != args.len() - 1 {
            buf.push_str(", ");
          }
        }

        buf
      }

      ErrorType::MissingValue(name) => format!("Missing value for flag \"{name}\""),
    };

    let mut context = String::new();
    if !cli.positional.is_empty() {
      context.push_str("Still waiting on positional args:\n");
      for arg in &cli.positional {
        context.push_str(&format!("- {}\n", arg.name));
      }
    }

    let required_named = cli
      .named
      .iter()
      .filter(|(_, arg)| arg.required)
      .collect::<Vec<_>>();
    if !required_named.is_empty() {
      context.push_str("Still waiting on named args:\n");
      for (name, _) in required_named {
        context.push_str(&format!("- {name}\n"));
      }
    }

    if context.is_empty() {
      cause
    } else {
      format!("{cause}\n{context}")
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use maplit::hashmap;

  fn s(st: &str) -> String {
    st.to_string()
  }

  fn cli() -> Cli {
    Cli::new()
      .positional("file")
      .named("verbose", false, false)
      .named("output-format", true, true)
  }

  #[test]
  fn cli_test1() {
    let args = vec![s("my-app"), s("--output-format"), s("json"), s("foo.rs")];
    assert_eq!(
      Ok(hashmap! {
        s("output-format") => s("json"),
        s("file") => s("foo.rs")
      }),
      cli().parse(args)
    );
  }

  #[test]
  fn cli_test2() {
    let args = vec![s("my-app")];
    assert_eq!(
      Err(s(r#"Missing positional arguments: file
Still waiting on positional args:
- file
Still waiting on named args:
- output-format
"#)),
      cli().parse(args)
    );
  }

  #[test]
  fn cli_test3() {
    let args = vec![
      s("my-app"),
      s("--verbose"),
      s("foo.rs"),
      s("--output-4mat"),
      s("json"),
    ];
    assert_eq!(
      Err(
        r#"Invalid named argument "output-4mat"
Still waiting on named args:
- output-format
"#
        .into()
      ),
      cli().parse(args)
    );
  }
}
