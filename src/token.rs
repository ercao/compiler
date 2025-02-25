use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
  Illegal,        // 不识别符号
  Ident(String),  // 标识符
  Integer(isize), // 整数

  //
  Bang,      // !
  Plus,      // +
  Minus,     // -
  Asterisk,  // *
  Slash,     // /
  Lt,        // <
  LtEq,      // <=
  Gt,        // >
  GtEq,      // >=
  Eq,        // ==
  Ne,        // !=
  AddAssign, // +=
  SubAssign, // -=
  MulAssign, // *=
  DivAssign, // /=
  Assign,    // =

  //
  Lparen,   // (
  Rparen,   // )
  Lbracket, // [
  Rbracket, // ]
  Lbrace,   // {
  Rbrace,   // }

  Comma,     // ,
  Semicolon, // ;

  //
  If,       // if
  Else,     // else
  While,    // while
  Const,    // const
  Var,      // var
  Function, // fn
  Return,   // return

  //
  EOF,
}

impl Token {
  /// 是否是语句开始符号
  #[rustfmt::skip]

  /// 是否是表达式的开始符号
  pub fn is_expression_begin(&self) -> bool {
    matches!(
      self,
      Self::Ident(_)
        | Self::Integer(_)
        | Self::Bang
        | Self::Minus
        | Self::Lparen
        | Self::Lbracket
        | Self::Lbrace
        | Self::If
        | Self::While
        | Self::Function
    )
  }
}

impl Display for Token {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Token::Illegal => "unknown",
      Token::Ident(_s) => _s,
      Token::Integer(_i) => "integer",
      Token::Plus => "+",
      Token::Minus => "-",
      Token::Asterisk => "*",
      Token::Slash => "/",
      Token::Eq => "==",
      Token::Ne => "!=",
      Token::Lt => "<",
      Token::LtEq => "<=",
      Token::Gt => ">",
      Token::GtEq => ">=",
      Token::Lparen => "(",
      Token::Rparen => ")",
      Token::Comma => ",",
      Token::Semicolon => ";",
      Token::Assign => "=",
      Token::If => "if",
      Token::While => "while",
      Token::Const => "const",
      Token::Var => "var",
      Token::Function => "fn",
      Token::Lbrace => "}",
      Token::Rbrace => "{",
      Token::Lbracket => "[",
      Token::Rbracket => "]",
      Token::Bang => "!",
      Token::Else => "else",
      Token::Return => "return",
      Token::AddAssign => "+=",
      Token::SubAssign => "-=",
      Token::MulAssign => "*=",
      Token::DivAssign => "/=",
      Token::EOF => "eof",
    })
  }
}
