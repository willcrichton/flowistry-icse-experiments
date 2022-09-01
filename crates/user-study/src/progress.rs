//! Iterator wrapper that prints out a progress bar.

pub struct ProgressBar<Iter> {
  index: usize,
  iter: Iter,
  bound: Option<usize>,
  brackets: Option<(char, char)>,
}

impl<Iter> ProgressBar<Iter>
where
  Iter: ExactSizeIterator,
{
  /// Create a progress bar for bounded iterators.
  pub fn new(iter: Iter) -> Self {
    let bound = Some(iter.len());
    ProgressBar {
      index: 0,
      iter,
      bound,
      brackets: Some(('[', ']')),
    }
  }
}

impl<Iter> ProgressBar<Iter> {
  /// Create a progress bar for unbounded iterators.
  pub fn new_unbounded(iter: Iter) -> Self {
    ProgressBar {
      index: 0,
      iter,
      bound: None,
      brackets: None,
    }
  }

  fn bump(&mut self) {
    self.index += 1;
  }
}

// Core logic for printing out the progress bar.
impl<Iter> Iterator for ProgressBar<Iter>
where
  Iter: Iterator,
{
  type Item = Iter::Item;

  fn next(&mut self) -> Option<Self::Item> {
    let mut output = String::new();

    self.bump();
    let stars = "*".repeat(self.index);

    match self.bound {
      None => {
        output.push_str(&stars);
      }

      Some(bound) => {
        let spacing = bound - self.index;
        let spaces = " ".repeat(spacing);

        match self.brackets {
          None => {
            output.push_str(&stars);
            output.push_str(&spaces);
          }

          Some(brackets) => {
            output.push(brackets.0);
            output.push_str(&stars);
            output.push_str(&spaces);
            output.push(brackets.1);
          }
        }
      }
    }
    println!("{output}");

    self.iter.next()
  }
}

#[test]
fn progress_test1() {
  for _ in ProgressBar::new_unbounded(0..).take(3) {}
}

#[test]
fn progress_test2() {
  for _ in ProgressBar::new(vec![1, 2, 3].into_iter()) {}
}
