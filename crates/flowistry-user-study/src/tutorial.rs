#![allow(dead_code)]
use std::{collections::HashSet, hash::Hash};

pub fn union<T: Hash + Clone + Eq>(set: &mut HashSet<T>, other: &HashSet<T>) -> bool {
  let orig_len = set.len();

  for el in other {
    set.insert(el.clone());
  }

  orig_len != set.len()
}

#[derive(Debug, PartialEq, Eq)]
pub struct Person {
  name: String,
  location: Location,
  admin: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Location {
  WorkingFromHome,
  OnSite,
}

impl Person {
  pub fn new(name: &str, location: &str) -> Option<Self> {
    let mut name = name.to_lowercase();
    let first = name.chars().next().unwrap();
    name.replace_range(
      0..first.len_utf8(),
      &first.to_uppercase().collect::<String>(),
    );

    let location = match location {
      "whf" | "WorkingFromHome" => Location::WorkingFromHome,
      "onsite" | "OnSite" => Location::OnSite,
      _ => {
        return None;
      }
    };

    let admin = name == "Will";

    Some(Person {
      name,
      location,
      admin,
    })
  }

  pub fn set_location(&mut self, location: Location) {
    self.location = location;
  }

  pub fn make_remote(&mut self) {
    let new_location = Location::WorkingFromHome;
    self.set_location(new_location);
    // self.location = new_location;

    println!("{} has become remote.", self.name);
  }
}

#[test]
fn tutorial_test1() {
  assert_eq!(
    Some(Person {
      name: "Will".to_string(),
      location: Location::WorkingFromHome,
      admin: true
    }),
    Person::new("will", "wfh")
  )
}
