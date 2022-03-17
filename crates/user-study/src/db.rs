use std::collections::HashMap;

/// Primitive data types of the database.
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

/// Data structure for efficiently finding which rows contain
/// a particular column value.
#[derive(Default)]
struct Index {
  index: HashMap<Field, Vec<usize>>,
}

impl Index {
  /// Updates the index after a new row is added to the database.
  pub fn update(&mut self, field: Field, row_index: usize) {
    self.index.insert(field, vec![row_index]);
  }

  /// Returns the indexes of rows that have the given field.
  pub fn lookup(&self, field: &Field) -> Option<&Vec<usize>> {
    self.index.get(field)
  }
}

pub struct Database {
  col_to_idx: HashMap<String, usize>,
  indexes: HashMap<String, Index>,
  rows: Vec<Row>,
}

impl Database {
  /// Creates a new database. `columns` is a list of names of columns,
  /// and `indexed` is a list of column names that the database should
  /// build indexes for.
  pub fn new(columns: Vec<String>, indexed: Vec<String>) -> Self {
    let indexes = indexed
      .into_iter()
      .map(|col| (col, Index::default()))
      .collect::<HashMap<_, _>>();

    let col_to_idx = columns
      .into_iter()
      .enumerate()
      .map(|(i, s)| (s, i))
      .collect::<HashMap<_, _>>();

    Database {
      indexes,
      col_to_idx,
      rows: Vec::new(),
    }
  }

  /// Inserts a row into the database, updating indexes as necessary.
  pub fn insert(&mut self, row: Row) {
    let row_index = self.rows.len();

    for (col, index) in self.indexes.iter_mut() {
      let field = &row.fields[self.col_to_idx[col]];
      index.update(field.clone(), row_index);
    }

    self.rows.push(row);
  }

  /// Finds the rows in the database whose `column` is equal to `field`.
  pub fn select(&self, column: &str, field: &Field) -> Vec<&Row> {
    if !self.col_to_idx.contains_key(column) {
      panic!("Invalid column: {column:?}");
    }

    match self.indexes.get(column) {
      Some(index) => match index.lookup(field) {
        Some(index_matching) => index_matching.iter().map(|i| &self.rows[*i]).collect(),
        None => Vec::new(),
      },

      None => {
        let col_idx = self.col_to_idx[column];
        self
          .rows
          .iter()
          .filter(|row| &row.fields[col_idx] == field)
          .collect::<Vec<_>>()
      }
    }
  }
}

#[test]
fn db_test1() {
  let mut db = Database::new(vec!["id".into(), "name".into()], vec![]);
  let row = Row::new(vec![Field::Integer(0), Field::String("will".into())]);
  db.insert(row.clone());
  assert_eq!(vec![&row], db.select("id", &Field::Integer(0)));
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
    db.select("name", &Field::String("will".into()))
  );
}
