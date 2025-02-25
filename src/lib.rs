#![feature(pattern)]

use std::fmt::Display;

pub mod ast;
pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod vm;

#[derive(Debug, Clone, Copy)]
pub struct SpanOffset {
  pub begin: usize,
  pub end: usize,
}

impl From<(usize, usize)> for SpanOffset {
  fn from(value: (usize, usize)) -> Self {
    SpanOffset { begin: value.0, end: value.1 }
  }
}

impl Display for SpanOffset {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{},{}", self.begin, self.end)
  }
}
