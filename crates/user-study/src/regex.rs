pub enum Regex {
  Char(char),
  And(Box<Regex>, Box<Regex>),
  Or(Box<Regex>, Box<Regex>),
  Repeat(Box<Regex>, usize),
}

impl Regex {
  pub fn matches(&self, s: impl AsRef<str>) -> bool {
    let s = s.as_ref();
    let chars = &mut s.chars();
    self.matches_chars(chars) && chars.next().is_none()
  }

  pub fn matches_chars(&self, chars: &mut (impl Iterator<Item = char> + Clone)) -> bool {
    match self {
      Regex::Char(c) => match chars.next() {
        Some(c2) => *c == c2,
        _ => false,
      },
      Regex::And(r1, r2) => r1.matches_chars(chars) && r2.matches_chars(chars),
      Regex::Or(r1, r2) => r1.matches_chars(chars) || r2.matches_chars(chars),
      Regex::Repeat(r, n) => (0..*n).all(|_| r.matches_chars(chars)),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  fn chr(c: char) -> Box<Regex> {
    Box::new(Regex::Char(c))
  }

  #[test]
  fn regex_test1() {
    let r = Regex::And(chr('a'), Box::new(Regex::Repeat(chr('b'), 3)));
    assert!(r.matches("abbb"));
    assert!(!r.matches("abb"));
  }

  #[test]
  fn regex_test2() {
    let r = Regex::And(
      Box::new(Regex::And(
        chr('a'),
        Box::new(Regex::Or(chr('b'), chr('c'))),
      )),
      chr('d'),
    );
    assert!(r.matches("abd"));
    assert!(r.matches("acd"));
    assert!(!r.matches("abcd"));
  }
}
