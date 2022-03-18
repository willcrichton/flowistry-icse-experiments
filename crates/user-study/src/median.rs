use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct Country {
  name: String,
  continent: String,
  population: usize,
  exports: Vec<String>,
}

/// For each continent, and for countries that have the given export,
/// returns the median population and name of countries at the median.
pub fn median_population(
  mut countries: &[Country],
  export: String,
) -> HashMap<String, (HashSet<String>, usize)> {
  let mut continents: HashMap<&str, Vec<_>> = HashMap::new();
  for country in countries {
    if country.exports.contains(&export) {
      continents
        .entry(country.continent.as_str())
        .or_default()
        .push(country);
    }
  }

  let mut medians = HashMap::new();
  for (continent, cont_countries) in continents.iter_mut() {
    cont_countries.sort_by_key(|c| c.population);
    let n = cont_countries.len();
    let median_countries = if n % 2 == 1 {
      vec![&cont_countries[(n - 1) / 2]]
    } else {
      vec![&cont_countries[n / 2 - 1], &cont_countries[n / 2]]
    };

    let names = median_countries
      .iter()
      .map(|c| c.name.to_string())
      .collect::<HashSet<_>>();

    let pop_total = median_countries.iter().map(|c| c.population).sum::<usize>();
    let median = pop_total / cont_countries.len();

    medians.insert(continent.to_string(), (names, median));
  }

  medians
}

#[test]
fn median_test1() {
  use maplit::{hashmap, hashset};

  let s = |s: &str| -> String { s.to_string() };
  let countries = &[
    Country {
      name: s("USA"),
      population: 328,
      continent: s("North America"),
      exports: vec![s("wheat"), s("cheese")],
    },
    Country {
      name: s("Canada"),
      population: 37,
      continent: s("North America"),
      exports: vec![s("poutine"), s("wheat")],
    },
    Country {
      name: s("Mexico"),
      population: 128,
      continent: s("North America"),
      exports: vec![s("steel"), s("wheat")],
    },
    Country {
      name: s("Costa Rica"),
      population: 5,
      continent: s("North America"),
      exports: vec![s("coffee"), s("bananas")],
    },
    Country {
      name: s("Ethiopia"),
      population: 109,
      continent: s("Africa"),
      exports: vec![s("coffee"), s("wheat")],
    },
  ];

  assert_eq!(
    hashmap! {
      s("North America") => (hashset![s("Mexico")], 128),
      s("Africa") => (hashset![s("Ethiopia")], 109)
    },
    median_population(countries, s("wheat"))
  );
}
