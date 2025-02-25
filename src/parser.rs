use std::{mem, result};

use crate::{
  ast::{AstNode, Expression, ExpressionKind, Identifier, Infix, Prefix, Program, Statement, StatementKind},
  lexer::Lexer,
  token::Token,
  SpanOffset,
};

type Result<T> = result::Result<T, (String, SpanOffset)>;

/// 语法解析器
/// 转换为抽象语法树
pub struct Paser<'a> {
  lexer: Lexer<'a>,

  prev_pos: SpanOffset,

  current_token: (Token, SpanOffset),
  next_token: (Token, SpanOffset),
}

impl<'a> Paser<'a> {
  pub fn new(input: &'a str) -> Self {
    let mut lexer = Lexer::new(input);

    let current_token = lexer.next();
    let next_token = lexer.next();

    Paser { lexer, prev_pos: (0, 0).into(), current_token, next_token }
  }

  pub fn paser(input: &'a str) -> Result<Program> {
    let mut paser = Paser::new(input);

    let mut statements = vec![];

    while paser.current_token.0 != Token::EOF {
      statements.push(paser.paser_statement()?);
    }
    Ok(Program { statements })
  }

  /// 解析语句
  fn paser_statement(&mut self) -> Result<Statement> {
    let pos = self.current_token.1;

    let kind = match self.current_token.0 {
      Token::Const => self.paser_const()?,
      Token::Var => self.paser_variable()?,
      Token::Function => self.paser_function()?,

      Token::Ident(_)
        if matches!(
          self.next_token.0,
          Token::Assign | Token::AddAssign | Token::SubAssign | Token::MulAssign | Token::DivAssign
        ) =>
      {
        if let (Token::Ident(ident), ident_pos) = self.next_token() {
          let (token, _) = self.next_token(); // 赋值token
          let pos = self.current_token.1;

          let expression = match &token {
            Token::Assign => self.paser_expression(Precedence::Lowest),

            token => {
              let mut expression = Expression {
                pos,
                kind: ExpressionKind::Infix(
                  match token {
                    Token::AddAssign => Infix::Add,
                    Token::SubAssign => Infix::Sub,
                    Token::MulAssign => Infix::Mul,
                    Token::DivAssign => Infix::Div,
                    _ => unreachable!(),
                  },
                  Box::new(Expression { pos: ident_pos, kind: ExpressionKind::Identifier(ident.clone()) }),
                  Box::new(self.paser_expression(Precedence::Lowest)?),
                ),
              };
              expression.pos.begin = pos.begin;
              expression.pos.end = self.prev_pos.end;

              Ok(expression)
            }
          }?;

          StatementKind::Assign(Identifier { pos: ident_pos, name: ident }, expression)
        } else {
          unreachable!()
        }
      }

      Token::Semicolon => StatementKind::Empty,
      Token::Return => {
        self.next_token();

        StatementKind::Return(match self.current_token.0 {
          Token::Semicolon => None,
          _ => Some(self.paser_expression(Precedence::Lowest)?),
        })
      }

      _ => StatementKind::Expression(self.paser_expression(Precedence::Lowest)?),
    };

    let end = self.prev_pos.end;

    while let Token::Semicolon = self.current_token.0 {
      self.next_token();
    }

    Ok(Statement { pos: SpanOffset { begin: pos.begin, end }, kind })
  }

  fn paser_const(&mut self) -> Result<StatementKind> {
    self.next_token();

    let mut constants = vec![];

    loop {
      let (token, pos) = self.next_token();
      if let Token::Ident(ident) = token {
        self.expect(Token::Assign);

        let (token, pos) = self.next_token();
        if let Token::Integer(value) = token {
          constants.push((Identifier { pos, name: ident }, Expression { pos, kind: ExpressionKind::Integer(value) }))
        } else {
          return Err((format!("only integer can assign to constant, but get {}", token), pos))?;
        }
      } else {
        return Err((format!("const define need identifier, but get {}", token), pos));
      }

      if Token::Comma == self.current_token.0 {
        self.next_token();
      } else {
        break;
      }
    }

    Ok(StatementKind::Const(constants))
  }

  fn paser_variable(&mut self) -> Result<StatementKind> {
    self.next_token();

    let mut variables = vec![];

    loop {
      let (token, pos) = self.next_token();
      if let Token::Ident(ident) = token {
        variables.push((
          Identifier { pos, name: ident },
          if matches!(self.current_token.0, Token::Comma | Token::Semicolon) {
            Expression { pos, kind: ExpressionKind::Integer(0) }
          } else {
            self.expect(Token::Assign);
            self.paser_expression(Precedence::Lowest)?
          },
        ));
      } else {
        return Err((format!("nead identifier, but get {}", token), pos));
      }

      if Token::Comma == self.current_token.0 {
        self.next_token();
      } else {
        break;
      }
    }

    Ok(StatementKind::Variable(variables))
  }

  ///
  /// 解析 function 定义语句
  ///
  fn paser_function(&mut self) -> Result<StatementKind> {
    self.next_token();
    let (token, pos) = self.next_token();

    if let Token::Ident(name) = token {
      self.expect(Token::Lparen);

      let mut args = vec![];
      if let (Token::Ident(_), ident_pos) = self.current_token {
        loop {
          let (token, pos) = self.next_token();
          if let Token::Ident(name) = token {
            args.push(Identifier { pos: ident_pos, name });
          } else {
            return Err((format!("function argument declare expect identifier, but get {}", token), pos));
          }

          if self.current_token.0 == Token::Comma {
            self.next_token();
          } else {
            break;
          }
        }
      }

      self.expect(Token::Rparen);
      self.expect(Token::Lbrace);

      Ok(StatementKind::Function(Identifier { pos, name }, args, self.parse_block_statement()?))
    } else {
      Err((format!("expect function identifier, but get {}", token), pos))
    }
  }

  ///
  /// 解析语句块
  ///
  fn parse_block_statement(&mut self) -> Result<Vec<Statement>> {
    let mut statements = vec![];
    while !matches!(self.current_token.0, Token::EOF | Token::Rbrace) {
      statements.push(self.paser_statement()?);
    }

    self.next_token();
    Ok(statements)
  }

  ///
  /// 解析表达式
  ///
  fn paser_expression(&mut self, precedence: Precedence) -> Result<Expression> {
    let (token, pos) = self.next_token();

    let kind = match token {
      Token::Integer(integer) => ExpressionKind::Integer(integer),
      Token::Bang => ExpressionKind::Prefix(Prefix::Not, Box::new(self.paser_expression(Precedence::Prefix)?)),
      Token::Minus => ExpressionKind::Prefix(Prefix::Neg, Box::new(self.paser_expression(Precedence::Prefix)?)),
      Token::Lparen => self.paser_group_expression(Token::Rparen)?,

      // Token::Lbracket => Expression::Array(self.paser_expressions(Token::Rbracket)),
      // Token::Lbrace => todo!("hash"),
      Token::If => self.paser_if()?,
      Token::While => self.paser_while()?,
      Token::Ident(ref ident) => ExpressionKind::Identifier(ident.to_string()),

      x => {
        return Err((format!("current position for this expression get unexpected token: {}", x), pos));
      }
    };

    let mut expression = Expression { pos: SpanOffset { begin: pos.begin, end: self.prev_pos.end }, kind };

    // 如果下一个运算符优先级
    while self.current_token.0 != Token::Semicolon && precedence < self.token_precedence(&self.current_token.0) {
      let (token, pos) = self.next_token();

      let kind = if let (p, Some(infix)) = self.infix_token(&token) {
        // 中缀表达式
        ExpressionKind::Infix(infix, Box::new(expression), Box::new(self.paser_expression(p)?))
      } else if token == Token::Lparen {
        // 函数调用表达式

        if let ExpressionKind::Identifier(ident) = &expression.kind {
          ExpressionKind::Call(
            Identifier { pos: expression.pos, name: ident.to_string() },
            self.paser_expressions(Token::Rparen)?,
          )
        } else {
          return Err((format!("only support call function ident, but get {}", expression.unparse()), pos));
        }
      } else {
        unreachable!()
      };

      expression = Expression { pos: SpanOffset { begin: pos.begin, end: self.current_token.1.end }, kind }
    }

    Ok(expression)
  }

  ///
  /// 解析 if 表达式
  ///
  fn paser_if(&mut self) -> Result<ExpressionKind> {
    let condition = self.paser_expression(Precedence::Lowest)?;

    self.expect(Token::Lbrace);

    Ok(ExpressionKind::If(
      Box::new(condition),
      self.parse_block_statement()?,
      if self.current_token.0 == Token::Else {
        self.next_token();
        self.expect(Token::Lbrace);

        Some(self.parse_block_statement()?)
      } else {
        None
      },
    ))
  }

  ///
  /// 解析 while 表达式
  ///
  fn paser_while(&mut self) -> Result<ExpressionKind> {
    let condition = self.paser_expression(Precedence::Lowest)?;

    self.expect(Token::Lbrace);

    Ok(ExpressionKind::While(Box::new(condition), self.parse_block_statement()?))
  }

  /// 解析 [] ()
  fn paser_group_expression(&mut self, close_token: Token) -> Result<ExpressionKind> {
    let e = self.paser_expression(Precedence::Lowest)?;
    self.expect(close_token);
    Ok(e.kind)
  }

  /// 解析 逗号分割的 表达式
  fn paser_expressions(&mut self, close_token: Token) -> Result<Vec<Expression>> {
    let mut expressions = vec![];

    if self.current_token.0.is_expression_begin() {
      loop {
        expressions.push(self.paser_expression(Precedence::Lowest)?);

        if self.current_token.0 == Token::Comma {
          self.next_token();
        } else {
          break;
        }
      }
    }

    self.expect(close_token);

    Ok(expressions)
  }

  ///
  /// 返回当前token 并 获取下一个 token
  ///
  fn next_token(&mut self) -> (Token, SpanOffset) {
    self.prev_pos = self.current_token.1;

    // println!("{:?}", self.current_token);
    mem::replace(&mut self.current_token, mem::replace(&mut self.next_token, self.lexer.next()))
  }

  /// 期待当前 token
  /// 并且获取下一个token
  fn expect(&mut self, token: Token) {
    if self.current_token.0 != token {
      panic!("expect {:?}, but get {:?}", token, self.current_token);
    }
    self.next_token();
  }

  fn token_precedence(&self, token: &Token) -> Precedence {
    if let (precedence, Some(_)) = self.infix_token(token) {
      precedence
    } else {
      match token {
        Token::Minus | Token::Bang => Precedence::Prefix,
        Token::Lparen => Precedence::Suffix,
        _ => Precedence::Lowest,
      }
    }
  }

  fn infix_token(&self, token: &Token) -> (Precedence, Option<Infix>) {
    match token {
      Token::Eq => (Precedence::Equals, Some(Infix::Eq)),
      Token::Ne => (Precedence::Equals, Some(Infix::Ne)),
      Token::Lt => (Precedence::LessGreater, Some(Infix::Lt)),
      Token::Gt => (Precedence::LessGreater, Some(Infix::Gt)),
      Token::LtEq => (Precedence::LessGreater, Some(Infix::LtEq)),
      Token::GtEq => (Precedence::LessGreater, Some(Infix::GtEq)),

      Token::Plus => (Precedence::Sum, Some(Infix::Add)),
      Token::Minus => (Precedence::Sum, Some(Infix::Sub)),
      Token::Asterisk => (Precedence::Product, Some(Infix::Mul)),
      Token::Slash => (Precedence::Product, Some(Infix::Div)),
      _ => (Precedence::Lowest, None),
    }
  }
}

/// 操作符优先级
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
  Lowest,
  Equals,      // 条件运算符
  LessGreater, // 关系运算符
  Sum,         // 加减运算符
  Product,     // 乘除运算符
  Prefix,      // 前缀运算符
  Suffix,      // 后缀表达式
}

#[cfg(test)]
mod tests {

  use crate::ast::AstNode;

  use super::Paser;

  #[test]
  fn test_infix_expression() {
    test_parsing(vec![
      // 赋值表达式
      ("a += -5", "a = (a + (-5));"),
      ("a += 1 + 2", "a = (a + (1 + 2));"),
      ("a = -5", "a = (-5);"),
      ("var a = 1 + 4 * 2 + 2", "var a = ((1 + (4 * 2)) + 2);"),
      // 算术表达式
      ("10 + -5", "(10 + (-5));"),
      ("-a * b;", "((-a) * b);"),
      ("!-a;", "(!(-a));"),
      ("a + b + c;", "((a + b) + c);"),
      ("a + b - c;", "((a + b) - c);"),
      ("a * b * c;", "((a * b) * c);"),
      ("a * b / c;", "((a * b) / c);"),
      ("a + b / c;", "(a + (b / c));"),
      ("a + b * c + d / e - f;", "(((a + (b * c)) + (d / e)) - f);"),
      ("3 + 4; -5 * 5;", "(3 + 4); ((-5) * 5);"),
      ("5 > 4 == 3 < 4;", "((5 > 4) == (3 < 4));"),
      ("5 < 4 != 3 > 4;", "((5 < 4) != (3 > 4));"),
      ("3 + 4 * 5 == 3 * 1 + 4 * 5;", "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)));"),
      ("1 + (2 + 3) + 4;", "((1 + (2 + 3)) + 4);"),
      ("(5 + 5) * 2;", "((5 + 5) * 2);"),
      ("2 / (5 + 5);", "(2 / (5 + 5));"),
      ("-(5 + 5);", "(-(5 + 5));"),
      // if 表达式
      ("if (x < y) { x; };", "if (x < y) { x; };"),
      ("if (x < y) { x; } else { y; };", "if (x < y) { x; } else { y; };"),
      // return 表达式
      ("return x;", "return x;"),
      ("return x; return 2 * 3;", "return x; return (2 * 3);"),
      ("return 2 * 4 + 5;", "return ((2 * 4) + 5);"),
      // 函数声明定义表达式
      ("fn xx(x) { x * 9; };", "fn xx(x) { (x * 9); }"),
      ("fn xx(x, y) { x + y; };", "fn xx(x, y) { (x + y); }"),
      // 函数调用表达式
      ("call();", "call();"),
      ("add(1, 2 * 3, 4 + 5);", "add(1, (2 * 3), (4 + 5));"),
      ("a + add(b * c) + d;", "((a + add((b * c))) + d);"),
      ("add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8));", "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)));"),
      ("add(a + b + c * d / f + g)", "add((((a + b) + ((c * d) / f)) + g));"),
      // 常量, 变量声明
      ("const x = 3;", "const x = 3;"),
      ("var x;", "var x = 0;"),
    ]);
  }

  fn test_parsing(t: Vec<(&str, &str)>) {
    for (input, unparse) in t {
      let program = Paser::paser(input).unwrap();

      // println!("{}", input);
      // (0..).take(input.len()).for_each(|x| print!("{}", x));
      // println!();
      //
      // println!("{}", program.unparse());
      // println!("{:#?}", program);

      assert_eq!(program.unparse(), unparse)
    }
  }
}
