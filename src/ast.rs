use std::fmt::Display;

use crate::SpanOffset;

pub trait AstNode {
  fn unparse(&self) -> String;
}

#[derive(Debug)]
pub struct Program {
  pub statements: Vec<Statement>,
}

impl AstNode for Program {
  fn unparse(&self) -> String {
    self.statements.unparse()
  }
}

#[derive(Debug)]
pub struct Statement {
  pub pos: SpanOffset,
  pub kind: StatementKind,
}

/// 标识符
#[derive(Debug)]
pub struct Identifier {
  pub pos: SpanOffset,

  pub name: String,
}

impl AstNode for Identifier {
  fn unparse(&self) -> String {
    self.name.clone()
  }
}

/// 语句
#[derive(Debug)]
pub enum StatementKind {
  Empty,                                                 // 空语句
  Const(Vec<(Identifier, Expression)>),                  // 常量声明语句
  Variable(Vec<(Identifier, Expression)>),               // 变量声明语句
  Function(Identifier, Vec<Identifier>, Vec<Statement>), // 函数语句
  Assign(Identifier, Expression),                        // 赋值语句
  Return(Option<Expression>),                            // 返回语句
  Expression(Expression),                                // 表达式语句
}

impl AstNode for [Statement] {
  fn unparse(&self) -> String {
    self.iter().map(AstNode::unparse).collect::<Vec<_>>().join(" ")
  }
}

impl AstNode for Statement {
  fn unparse(&self) -> String {
    match &self.kind {
      StatementKind::Empty => ";".to_string(),
      StatementKind::Const(c) => format!(
        "const {};",
        c.iter().map(|(a, b)| format!("{} = {}", a.unparse(), b.unparse())).collect::<Vec<_>>().join(", ")
      ),
      StatementKind::Variable(vs) => format!(
        "var {:};",
        vs.iter().map(|(a, b)| format!("{:} = {:}", a.unparse(), b.unparse())).collect::<Vec<_>>().join(", ")
      ),

      StatementKind::Function(name, args, b) => {
        format!(
          "fn {}({}) {{ {} }}",
          name.unparse(),
          args.iter().map(AstNode::unparse).collect::<Vec<_>>().join(", "),
          b.unparse()
        )
      }

      StatementKind::Assign(ident, expression) => format!("{} = {};", ident.unparse(), expression.unparse()),

      StatementKind::Return(Some(e)) => format!("return {:};", e.unparse()),
      StatementKind::Return(None) => "return;".to_string(),
      StatementKind::Expression(e) => format!("{};", e.unparse()),
    }
  }
}

/// 表达式
#[derive(Debug)]
pub struct Expression {
  pub pos: SpanOffset,

  pub kind: ExpressionKind,
}

impl AstNode for Expression {
  fn unparse(&self) -> String {
    match &self.kind {
      ExpressionKind::Identifier(x) => x.to_string(),
      ExpressionKind::Integer(x) => format!("{:}", x),
      ExpressionKind::Infix(infix, left, right) => {
        format!("({:} {:} {:})", left.unparse(), infix, right.unparse())
      }
      ExpressionKind::Prefix(prefix, e) => format!("({:}{:})", prefix, e.unparse()),
      ExpressionKind::Call(ident, args) => {
        format!("{:}({:})", ident.unparse(), args.iter().map(|kind| kind.unparse()).collect::<Vec<_>>().join(", "),)
      }
      ExpressionKind::If(condition, t, e) => {
        let mut string = format!("if {:} {{ {:} }}", condition.unparse(), t.unparse());

        if let Some(x) = e {
          string.push_str(&format!(" else {{ {:} }}", x.unparse()))
        }
        string
      }
      ExpressionKind::While(c, b) => format!("while {:} {{ {:} }}", c.unparse(), b.unparse()),
    }
  }
}

/// 表达式
#[derive(Debug)]
pub enum ExpressionKind {
  Identifier(String),
  Integer(isize),
  Infix(Infix, Box<Expression>, Box<Expression>),
  Prefix(Prefix, Box<Expression>),
  Call(Identifier, Vec<Expression>),
  If(Box<Expression>, Vec<Statement>, Option<Vec<Statement>>), // if 语句
  While(Box<Expression>, Vec<Statement>),                      // while 语句
}

/// 前缀表达式
#[derive(Debug)]
pub enum Prefix {
  Not, // 取返
  Neg, // 取相反数
}

impl Display for Prefix {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Prefix::Not => write!(f, "!"),
      Prefix::Neg => write!(f, "-"),
    }
  }
}

/// 中缀表达式
#[derive(Debug)]
pub enum Infix {
  Eq,   // ==
  Ne,   // !=
  Lt,   // <
  Gt,   // >
  LtEq, // <=
  GtEq, // >=

  Add, // +
  Sub, // -
  Mul, // *
  Div, // /
}

impl Display for Infix {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Infix::Eq => write!(f, "=="),
      Infix::Ne => write!(f, "!="),
      Infix::Lt => write!(f, "<"),
      Infix::Gt => write!(f, ">"),
      Infix::LtEq => write!(f, "<="),
      Infix::GtEq => write!(f, ">="),
      Infix::Add => write!(f, "+"),
      Infix::Sub => write!(f, "-"),
      Infix::Mul => write!(f, "*"),
      Infix::Div => write!(f, "/"),
    }
  }
}
