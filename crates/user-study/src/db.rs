use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum Field {
  String(String),
  Integer(isize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
  fields: Vec<Field>,
}

impl Row {
  pub fn new(fields: Vec<Field>) -> Self {
    Row { fields }
  }
}

#[derive(Default)]
struct Index {
  index: HashMap<Field, HashSet<usize>>,
}

impl Index {
  pub fn update(&mut self, field: Field, row_index: usize) {
    self.index.insert(field, maplit::hashset! { row_index });
  }

  pub fn lookup(&self, field: &Field) -> Option<&HashSet<usize>> {
    self.index.get(field)
  }
}

pub struct Database {
  columns: Vec<String>,
  col_indexes: HashMap<String, usize>,
  indexes: HashMap<String, Index>,
  rows: Vec<Row>,
}

impl Database {
  pub fn new(columns: Vec<String>, indexed: Vec<String>) -> Self {
    let indexes = indexed
      .into_iter()
      .map(|col| (col, Index::default()))
      .collect::<HashMap<_, _>>();
    let col_indexes = columns
      .iter()
      .enumerate()
      .map(|(i, s)| (s.clone(), i))
      .collect::<HashMap<_, _>>();
    Database {
      columns,
      indexes,
      col_indexes,
      rows: Vec::new(),
    }
  }

  pub fn insert(&mut self, row: Row) {
    let row_index = self.rows.len();
    for (col, index) in self.indexes.iter_mut() {
      let field = &row.fields[self.col_indexes[col]];
      index.update(field.clone(), row_index);
    }
    self.rows.push(row);
  }

  pub fn select(&self, filter: &[(&str, &Field)]) -> Vec<&Row> {
    let mut matching = (0..self.rows.len()).collect::<HashSet<_>>();

    for (key, field) in filter {
      match self.indexes.get(*key) {
        Some(index) => match index.lookup(field) {
          Some(index_matching) => {
            matching = &matching & index_matching;
          }
          None => {
            return Vec::new();
          }
        },

        None => {
          let col_idx = self.col_indexes[*key];
          matching.retain(|row_idx| {
            let row = &self.rows[*row_idx];
            &row.fields[col_idx] == *field
          });
        }
      }
    }

    let mut matching = Vec::from_iter(matching);
    matching.sort();
    matching.into_iter().map(|i| &self.rows[i]).collect()
  }
}

#[test]
fn db_test1() {
  let mut db = Database::new(vec!["id".into(), "name".into()], vec!["id".into()]);
  let row = Row::new(vec![Field::Integer(0), Field::String("will".into())]);
  db.insert(row.clone());
  assert_eq!(vec![&row], db.select(&[("id", &Field::Integer(0))]));
}

#[test]
fn db_test2() {
  let mut db = Database::new(vec!["id".into(), "name".into()], vec!["name".into()]);
  let rows = [(0, "will"), (1, "connor"), (2, "maryyann"), (3, "will")]
    .into_iter()
    .map(|(id, name)| Row::new(vec![Field::Integer(id), Field::String(name.to_string())]))
    .collect::<Vec<_>>();
  for row in &rows {
    db.insert(row.clone());
  }
  assert_eq!(
    vec![&rows[0], &rows[3]],
    db.select(&[("name", &Field::String("will".into()))])
  );
}
