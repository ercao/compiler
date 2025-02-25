use std::str::pattern::{Pattern, SearchStep, Searcher};

use crate::{token::Token, SpanOffset};

pub struct Lexer<'a> {
  string: &'a str,
  cursor: usize, // 字符串位置
  offset: usize,
}

impl<'a> Lexer<'a> {
  pub fn new(string: &'a str) -> Self {
    Lexer { string, cursor: 0, offset: 0 }
  }

  /// 获取下一个 token
  ///
  /// 返回 (token, 位置)
  pub fn next(&mut self) -> (Token, SpanOffset) {
    self.offset += self.eat_whitespace().chars().count();

    let pos = self.cursor;
    let token = match self.peek_char() {
      None => Token::EOF,
      Some(ch) => {
        match ch {
          'a'..='z' | 'A'..='Z' | '_' => match self.eat_while(char::is_alphanumeric) {
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "const" => Token::Const,
            "var" => Token::Var,
            "fn" => Token::Function,
            "return" => Token::Return,
            s => Token::Ident(s.to_string()),
          },

          '0'..='9' => {
            // todo: 处理越界
            Token::Integer(self.eat_while(|ch| matches!(ch, '0'..='9')).parse().unwrap())
          }

          '=' if self.eat_if("==") => Token::Eq,
          '!' if self.eat_if("!=") => Token::Ne,
          '+' if self.eat_if("+=") => Token::AddAssign,
          '-' if self.eat_if("-=") => Token::SubAssign,
          '*' if self.eat_if("*=") => Token::MulAssign,
          '/' if self.eat_if("/=") => Token::DivAssign,
          '<' if self.eat_if("<=") => Token::LtEq,
          '>' if self.eat_if(">=") => Token::GtEq,
          '/' if self.eat_if("//") => {
            // Token::LineComment(self.eat_until('\n').to_string())
            self.offset += self.eat_until('\n').chars().count() + 2;
            return self.next();
          }

          ch => {
            self.eat();

            match ch {
              '!' => Token::Bang,
              '+' => Token::Plus,
              '-' => Token::Minus,
              '*' => Token::Asterisk,
              '/' => Token::Slash,
              '<' => Token::Lt,
              '>' => Token::Gt,
              '=' => Token::Assign,
              '(' => Token::Lparen,
              ')' => Token::Rparen,
              '[' => Token::Lbracket,
              ']' => Token::Rbracket,
              '{' => Token::Lbrace,
              '}' => Token::Rbrace,
              ',' => Token::Comma,
              ';' => Token::Semicolon,
              _ => {
                println!("{}", ch);
                unreachable!()
              }
            }
          }
        }
      }
    };

    let begin = self.offset;
    self.offset += self.string[pos..self.cursor].chars().count();
    (token, SpanOffset { begin, end: self.offset })
  }

  #[inline]
  pub fn cursor(&self) -> usize {
    self.cursor
  }

  /// 是否分词结束
  pub fn done(&self) -> bool {
    self.cursor == self.string.len()
  }

  pub fn after(&self) -> &'a str {
    unsafe { self.string.get_unchecked(self.cursor..) }
  }

  /// start: start <= cursor
  pub fn from(&self, start: usize) -> &'a str {
    unsafe { self.string.get_unchecked(start..self.cursor) }
  }

  /// 查看当前游标的下一个单词
  fn peek_char(&self) -> Option<char> {
    self.after().chars().next()
  }

  fn eat(&mut self) -> Option<char> {
    let peek = self.peek_char();
    if let Some(c) = peek {
      self.cursor += c.len_utf8();
    }
    peek
  }

  fn eat_if(&mut self, pattern: impl Pattern) -> bool {
    let start = self.cursor;

    match pattern.into_searcher(self.after()).next() {
      SearchStep::Match(_, end) => {
        self.cursor = start + end;
        true
      }
      _ => false,
    }
  }

  fn eat_until(&mut self, pattern: impl Pattern) -> &'a str {
    let start = self.cursor;

    self.cursor = match pattern.into_searcher(self.after()).next_match() {
      Some((end, _)) => start + end,
      None => self.string.len(),
    };

    self.from(start)
  }

  fn eat_while(&mut self, pattern: impl Pattern) -> &'a str {
    let start = self.cursor;
    let mut searcher = pattern.into_searcher(self.after());

    while let SearchStep::Match(_, end) = searcher.next() {
      self.cursor = start + end;
    }

    self.from(start)
  }

  fn eat_whitespace(&mut self) -> &'a str {
    self.eat_while(char::is_whitespace)
  }
}

#[cfg(test)]
mod tests {
  use crate::token::Token;

  use super::Lexer;

  #[test]
  fn test_char() {
    let mut lexer = Lexer::new(
      r" // 你好
    < > ! + - * / == != += -= *= /= () [ ] { } , ;",
    );
    for token in [
      Token::Lt,
      Token::Gt,
      Token::Bang,
      Token::Plus,
      Token::Minus,
      Token::Asterisk,
      Token::Slash,
      Token::Eq,
      Token::Ne,
      Token::AddAssign,
      Token::SubAssign,
      Token::MulAssign,
      Token::DivAssign,
      Token::Lparen,
      Token::Rparen,
      Token::Lbracket,
      Token::Rbracket,
      Token::Lbrace,
      Token::Rbrace,
      Token::Comma,
      Token::Semicolon,
    ] {
      assert_eq!(lexer.next().0, token)
    }
  }

  #[test]
  fn test_keyworld() {
    let mut lexer = Lexer::new("if while const var fn");
    for token in [Token::If, Token::While, Token::Const, Token::Var, Token::Function] {
      assert_eq!(lexer.next().0, token);
    }
  }

  #[test]
  fn test() {
    let src = "fdas";

    let mut lexer = Lexer::new(src);

    loop {
      let token = lexer.next();
      if token.0 == Token::EOF {
        break;
      }

      println!("{:?}", token);
    }

    // assert!(matches!(lexer.next(), (Token::Integer(41324), _, _)));
    // println!("{:?}", lexer.next());
    // println!("{:?}", lexer.next());
    // println!("{:?}", lexer.next());
    // println!("{:?}", lexer.next());
    // assert_eq!(lexer.next().0, Token::Integer(41324));
    // assert_eq!(lexer.next().0, Token::Ident("hello".to_string()));
    // assert_eq!(lexer.next().0, Token::Integer(12));
    // assert_eq!(lexer.next().0, Token::EOF);
    // assert_eq!(lexer.next().0, Token::EOF);

    // assert!(lexer.done())
  }
}
