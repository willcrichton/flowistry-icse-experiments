//! A tabular data processing program. See the doc comment below for details.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct Country {
  name: String,
  continent: String,
  population: usize,
  exports: BTreeSet<String>,
}

/// For each continent, and for countries that have the given export,
/// returns the median population and the name of countries forming the median.
///
/// For an odd number of elements, the median is the middle value.
/// For an even number of elements, the median is the average of the two middle values.
pub fn median_population(
  countries: &[Country],
  export: String,
) -> BTreeMap<String, (BTreeSet<String>, usize)> {
  let mut continents: BTreeMap<&str, Vec<_>> = BTreeMap::new();
  for country in countries {
    if country.exports.contains(&export) {
      continents
        .entry(country.continent.as_str())
        .or_default()
        .push(country);
    }
  }

  let mut medians = BTreeMap::new();
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
      .collect::<BTreeSet<_>>();

    let pop_total = median_countries.iter().map(|c| c.population).sum::<usize>();
    let median = pop_total / cont_countries.len();

    medians.insert(continent.to_string(), (names, median));
  }

  medians
}

#[test]
fn median_test1() {
  use maplit::{btreemap, btreeset};

  let s = |s: &str| -> String { s.to_string() };
  let countries = &[
    Country {
      name: s("USA"),
      population: 328,
      continent: s("North America"),
      exports: btreeset![s("cheese"), s("wheat")],
    },
    Country {
      name: s("Canada"),
      population: 37,
      continent: s("North America"),
      exports: btreeset![s("poutine"), s("wheat")],
    },
    Country {
      name: s("Mexico"),
      population: 128,
      continent: s("North America"),
      exports: btreeset![s("steel"), s("wheat")],
    },
    Country {
      name: s("Costa Rica"),
      population: 5,
      continent: s("North America"),
      exports: btreeset![s("coffee"), s("bananas")],
    },
    Country {
      name: s("Tanzania"),
      population: 59,
      continent: s("Africa"),
      exports: btreeset![s("gold"), s("wheat")],
    },
    Country {
      name: s("Liberia"),
      population: 5,
      continent: s("Africa"),
      exports: btreeset![s("rubber"), s("wheat")],
    },
  ];

  assert_eq!(
    btreemap! {
      s("North America") => (btreeset![s("Mexico")], 128),
      s("Africa") => (btreeset![s("Liberia"), s("Tanzania")], 32)
    },
    median_population(countries, s("wheat"))
  );
}
