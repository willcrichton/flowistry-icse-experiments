use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Const {
  Num(isize),
  Str(String),
  Bool(bool),
  Tuple(Vec<Const>),
  Sum(Direction, Box<Const>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
  Left,
  Right,
}

impl Const {
  pub fn as_num(&self) -> isize {
    match self {
      Const::Num(n) => *n,
      _ => panic!("Could not convert {self:?} to num"),
    }
  }

  pub fn as_str(&self) -> &str {
    match self {
      Const::Str(s) => &*s,
      _ => panic!("Could not convert {self:?} to str"),
    }
  }

  pub fn as_bool(&self) -> bool {
    match self {
      Const::Bool(b) => *b,
      _ => panic!("Could not convert {self:?} to bool"),
    }
  }

  pub fn as_tuple(&self) -> &[Const] {
    match self {
      Const::Tuple(t) => &*t,
      _ => panic!("Could not convert {self:?} to tuple"),
    }
  }

  pub fn as_sum(&self) -> (&Direction, &Const) {
    match self {
      Const::Sum(d, c) => (d, c),
      _ => panic!("Could not convert {self:?} to sum"),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Binop {
  Add,
  Sub,
  Mul,
  Div,
}

#[derive(Debug, Clone)]
pub enum Relop {
  And,
  Or,
  Xor,
  Greater,
  Lesser,
}

#[derive(Debug, Clone)]
pub enum Expr {
  Const(Const),
  Var(String),
  Binop(Binop, Box<Expr>, Box<Expr>),
  Relop(Relop, Box<Expr>, Box<Expr>),
  If(Box<Expr>, Box<Expr>, Box<Expr>),
  Let(String, Box<Expr>, Box<Expr>),
  Tuple(Vec<Box<Expr>>),
  Proj(Box<Expr>, Box<Expr>),
  Sum(Direction, Box<Expr>),
  Match(Box<Expr>, (String, Box<Expr>), (String, Box<Expr>)),
}

#[derive(Default, Debug)]
pub struct State(HashMap<String, Const>);

impl Expr {
  pub fn eval(&self, state: &mut State) -> Const {
    println!("{self:?}\n  evaluating in state\n  {state:?}\n");

    let cnst = match self {
      Expr::Const(cnst) => cnst.clone(),

      Expr::Var(x) => state.0.get(x).unwrap().clone(),

      Expr::Binop(op, e1, e2) => {
        let n1 = e1.eval(state).as_num();
        let n2 = e2.eval(state).as_num();
        let n3 = match op {
          Binop::Add => n1 + n2,
          Binop::Sub => n1 - n2,
          Binop::Mul => n1 * n2,
          Binop::Div => n1 / n2,
        };
        Const::Num(n3)
      }

      Expr::Relop(op, e1, e2) => {
        let c1 = e1.eval(state);
        let c2 = e2.eval(state);
        let b3 = match op {
          Relop::And => c1.as_bool() && c2.as_bool(),
          Relop::Or => c1.as_bool() || c2.as_bool(),
          Relop::Xor => c1.as_bool() ^ c2.as_bool(),
          Relop::Greater => c1.as_num() > c2.as_num(),
          Relop::Lesser => c1.as_num() < c2.as_num(),
        };
        Const::Bool(b3)
      }

      Expr::If(cond, then_, else_) => {
        let b = cond.eval(state).as_bool();
        if b {
          then_.eval(state)
        } else {
          else_.eval(state)
        }
      }

      Expr::Let(x, x_expr, body) => {
        let x_const = x_expr.eval(state);
        state.0.insert(x.clone(), x_const);
        let v = body.eval(state);
        state.0.remove(x);
        v
      }

      Expr::Tuple(es) => {
        let vs = es.iter().map(|e| e.eval(state)).collect::<Vec<_>>();
        Const::Tuple(vs)
      }

      Expr::Proj(e1, e2) => {
        let v1 = e1.eval(state);
        let t = v1.as_tuple();
        let n = e2.eval(state).as_num();
        t[n as usize].clone()
      }

      Expr::Sum(d, e) => {
        let v = e.eval(state);
        Const::Sum(d.clone(), Box::new(v))
      }

      Expr::Match(discr, (x1, e1), (x2, e2)) => {
        let d = discr.eval(state);
        let (dir, c) = d.as_sum();
        let (x, e) = match dir {
          Direction::Left => (x1, e1),
          Direction::Right => (x2, e2),
        };
        state.0.insert(x.clone(), c.clone());
        let c2 = e.eval(state);
        state.0.remove(x);
        c2
      }
    };

    println!("{self:?}\n  stepped to {cnst:?}\n  with new state\n  {state:?}\n");
    cnst
  }
}

#[cfg(test)]
mod test {
  use super::*;

  fn num(n: isize) -> Box<Expr> {
    Box::new(Expr::Const(Const::Num(n)))
  }

  #[test]
  fn eval_test1() {
    let e = Expr::Relop(
      Relop::And,
      Box::new(Expr::Const(Const::Bool(true))),
      Box::new(Expr::Const(Const::Bool(false))),
    );
    assert_eq!(Const::Bool(false), e.eval(&mut State::default()));
  }

  #[test]
  fn eval_test2() {
    let e = Expr::Let(
      "x".into(),
      num(1),
      Box::new(Expr::Binop(
        Binop::Add,
        Box::new(Expr::Var("x".into())),
        num(2),
      )),
    );
    assert_eq!(Const::Num(3), e.eval(&mut State::default()));
  }

  #[test]
  fn eval_test3() {
    let x = "x".to_string();
    let e = Expr::Let(
      x.clone(),
      num(0),
      Box::new(Expr::Binop(
        Binop::Add,
        Box::new(Expr::Let(x.clone(), num(1), Box::new(Expr::Var(x.clone())))),
        Box::new(Expr::Var(x.clone())),
      )),
    );
    assert_eq!(Const::Num(1), e.eval(&mut State::default()));
  }
}
