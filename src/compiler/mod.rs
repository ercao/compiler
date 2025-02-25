use std::result;

use crate::{
  ast::{AstNode, Expression, ExpressionKind, Infix, Prefix, Program, Statement, StatementKind},
  vm::{self, builtins::Builtins, Opcode},
  SpanOffset,
};

use self::nametab::NameTable;

mod nametab;

type Error = (String, SpanOffset);
type Result<T> = result::Result<T, Vec<Error>>;

///
/// 语义分析 与 目标代码生成
///
pub struct Compiler {
  nametable: NameTable,
  builtins: Builtins,

  codes: Vec<Opcode>,
  cp: usize,

  errors: Vec<Error>,
}

impl Compiler {
  pub fn new() -> Self {
    Compiler { nametable: NameTable::new(), builtins: Builtins::new(), codes: vec![], cp: 0, errors: vec![] }
  }

  /// 编译 AST
  pub fn compile(program: &Program) -> Result<Vec<Opcode>> {
    let mut compiler = Compiler::new();

    let mut dx = 3;

    let cx_inte = compiler.gen_empty_code();
    compiler.gen_code(Opcode::Lit(0)); // 程序默认返回 0

    for statement in &program.statements {
      compiler.compile_statement(statement, 0, &mut dx);
    }

    if compiler.errors.is_empty() {
      compiler.gen_code(Opcode::Ret);
      compiler.codes[cx_inte] = Opcode::Int(dx);

      Ok(compiler.codes)
    } else {
      Err(compiler.errors)
    }
  }

  ///
  /// 编译语句块
  ///
  pub fn compile_block_statement(&mut self, statements: &Vec<Statement>, level: usize) {
    let mut dx = 1; // 位置
    let tx0 = self.nametable.tx();

    self.gen_code(Opcode::EnterScope);
    let cx_ine = self.gen_empty_code();

    for statement in statements {
      self.compile_statement(statement, level + 1, &mut dx);
    }

    self.gen_code(Opcode::LeaveScope);
    self.codes[cx_ine] = Opcode::Int(dx);

    self.nametable.rollback(self.nametable.tx() - tx0);
  }

  ///
  /// 编译语句
  ///
  pub fn compile_statement(&mut self, statement: &Statement, level: usize, dx: &mut usize) {
    match &statement.kind {
      StatementKind::Empty => {}
      StatementKind::Const(constants) => {
        for (ident, e) in constants {
          match e.kind {
            ExpressionKind::Integer(value) => {
              self.nametable.add_const(&ident.name, level, value);
            }
            _ => {
              self.errors.push((format!("only integer can assign to constant, but get {:?}", e.kind), e.pos));
            }
          }
        }
      }

      StatementKind::Variable(variable) => {
        for (ident, e) in variable {
          self.compile_expression(&e, level);

          self.nametable.add_variable(&ident.name, level, *dx as isize);

          self.gen_code(vm::Opcode::Sto(0, *dx as isize));
          *dx += 1;
        }
      }

      StatementKind::Function(ident, args, statements) => {
        if let Some(_) = self.builtins.lookup(&ident.name) {
          self.errors.push((
            format!("unable define funcation name as same as builtins function: {}", ident.name),
            statement.pos,
          ));

          return;
        }

        let cx_jmp = self.gen_empty_code();

        self.nametable.add_proceduce(&ident.name, level);
        let tx0 = self.nametable.tx();

        // 调用函数的时候, 参数在调用方压栈, 相对基地址为负数
        args.iter().enumerate().for_each(|(index, ident)| {
          self.nametable.add_variable(&ident.name, level + 1, -1 - index as isize);
        });

        let mut dx = 3;

        self.nametable.items[tx0].value = self.cp as isize;
        let cx_inte = self.gen_empty_code();
        self.gen_code(Opcode::Lit(0)); // 默认返回 0

        // 解析代码块
        for statement in statements {
          self.compile_statement(statement, level + 1, &mut dx);
        }

        self.gen_code(vm::Opcode::Ret);
        self.codes[cx_inte] = vm::Opcode::Int(dx);
        self.codes[cx_jmp] = vm::Opcode::Jmp(self.cp);

        // self.nametable.print_nametable();
        // 编译函数后，清理符号表
        self.nametable.rollback(self.nametable.tx() - tx0);
      }

      StatementKind::Assign(ident, expression) => {
        self.compile_expression(&expression, level);

        match self.nametable.find_kind(&ident.name, nametab::NameTableKind::Variable) {
          Some(item) => {
            self.gen_code(vm::Opcode::Sto(level - item.level, item.addr));
          }
          None => {
            self.errors.push((format!("variable is undefined: {:}", ident.name), statement.pos));
          }
        }
      }

      StatementKind::Return(e) => {
        if let Some(e) = e {
          self.compile_expression(&e, level);
        } else {
          self.gen_code(vm::Opcode::Lit(0)); // 默认返回 0
        }

        // 如果没有就返回 0
        self.gen_code(vm::Opcode::Ret);
      }

      StatementKind::Expression(e) => self.compile_expression(&e, level),
    }
  }

  /// 编译表达式
  pub fn compile_expression(&mut self, expression: &Expression, level: usize) {
    match &expression.kind {
      ExpressionKind::Identifier(ident) => match self.nametable.find(&ident) {
        Some(item) => {
          match item.kind {
            nametab::NameTableKind::Constant | nametab::NameTableKind::Proceduce => {
              self.gen_code(vm::Opcode::Lit(item.value))
            }
            nametab::NameTableKind::Variable => self.gen_code(vm::Opcode::Lod(level - item.level, item.addr)),
          };
        }

        None => {
          self.errors.push((format!("identifier is not define: {}", ident), expression.pos));
        }
      },

      ExpressionKind::Integer(integer) => self.gen_code(vm::Opcode::Lit(*integer)),
      ExpressionKind::Infix(infix, left, right) => {
        self.compile_expression(&left, level);
        self.compile_expression(&right, level);

        self.gen_code(match infix {
          Infix::Add => Opcode::Add,
          Infix::Sub => Opcode::Sub,
          Infix::Mul => Opcode::Mul,
          Infix::Div => Opcode::Div,

          Infix::Eq => Opcode::Eq,
          Infix::Ne => Opcode::Ne,
          Infix::Lt => Opcode::Lt,
          Infix::Gt => Opcode::Gt,
          Infix::LtEq => Opcode::Le,
          Infix::GtEq => Opcode::Ge,
        });
      }

      ExpressionKind::Prefix(prefix, e) => {
        match prefix {
          Prefix::Not => {
            self.compile_expression(&e, level);
            self.gen_code(Opcode::Not)
          }
          Prefix::Neg => {
            // 相反数
            self.gen_code(Opcode::Lit(0));
            self.compile_expression(&e, level);
            self.gen_code(Opcode::Sub)
          }
        };
      }

      // 参数压栈: 逆序压栈
      ExpressionKind::Call(indent, args) => {
        if let Some(id) = self.builtins.lookup(&indent.name) {
          // 调用内建函数
          for e in args.iter().rev() {
            self.compile_expression(e, level);
          }

          self.gen_code(vm::Opcode::Builtin(id, args.len()));
        } else {
          // 调用自定义函数
          match self.nametable.find_kind(&indent.name, nametab::NameTableKind::Proceduce) {
            Some(item) => {
              let (rlevel, addr) = (level - item.level, item.value);
              for e in args.iter().rev() {
                self.compile_expression(e, level);
              }

              self.gen_code(vm::Opcode::Lit(addr));
              self.gen_code(vm::Opcode::Cal(rlevel)); // todo 静态链
              self.gen_code(vm::Opcode::CallClean(args.len()));
            }

            None => {
              self.errors.push((format!("only support call function ident, but get {}", indent.unparse()), indent.pos));
            }
          }
        }
      }

      ExpressionKind::If(condition, then_s, else_s) => {
        self.compile_expression(&condition, level);

        let cx_jpc = self.gen_empty_code();

        self.gen_code(vm::Opcode::Lit(0));
        self.compile_block_statement(&then_s, level);
        self.codes[cx_jpc] = vm::Opcode::Jpc(self.cp);

        if let Some(e) = else_s {
          let cx_jmp = self.gen_empty_code();
          self.gen_code(vm::Opcode::Lit(0));
          self.codes[cx_jpc] = vm::Opcode::Jpc(self.cp);

          self.compile_block_statement(&e, level);
          self.codes[cx_jmp] = vm::Opcode::Jmp(self.cp)
        }
      }

      ExpressionKind::While(condition, s) => {
        let cx0 = self.cp;
        self.compile_expression(&condition, level);

        let cx_jpc = self.gen_empty_code();

        self.compile_block_statement(s, level);
        self.gen_code(vm::Opcode::Jmp(cx0));
        self.codes[cx_jpc] = vm::Opcode::Jpc(self.cp);
        self.gen_code(vm::Opcode::Lit(0));
      }
    }
  }

  /// 生成虚拟机指令
  fn gen_code(&mut self, opcode: vm::Opcode) {
    self.codes.push(opcode);
    self.cp += 1;
  }

  /// 生成空指令
  fn gen_empty_code(&mut self) -> usize {
    self.gen_code(vm::Opcode::None);
    self.cp - 1
  }
}

impl Default for Compiler {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use crate::{ast::AstNode, compiler::Compiler, parser::Paser, vm::VM};

  #[test]
  #[rustfmt::skip]
  fn test_block() {
    test_eval(vec![
      ( "fn func(n) { if n <= 1 { 1 } else { func(n - 1) + func(n - 2) } }; func(10);", 89 ),
      ( "if 2 <= 1 { 1 } else { 2 }", 2 ),
    ]);
  }

  #[test]
  fn test_statement() {
    test_eval(vec![
      // 空语句
      (";;;", 0),
      // 常量声明语句
      ("const a = 10, b = 20; a + b", 30),
      ("const a = 4; a", 4),
      // 变量声明语句
      ("var a; a", 0),
      ("var a = 4; a", 4),
      ("var x, y; x + y", 0),
      ("var x = 1, y = 2; x + y ", 3),
      ("var x = 1, y; x + y", 1),
      // 赋值语句
      ("var x = 1; x += 3;", 4),
      ("var x = 1; x += 3; x", 4),
      ("var x = 1; x -= 3; x", -2),
      ("var x = 1; x *= 3; x", 3),
      ("var x = 6; x /= 3; x", 2),
      // 返回语句
      ("return 1 + 2;", 3),
      ("return;", 0),
    ])
  }

  #[test]
  fn test_expression() {
    test_eval(vec![
      // 整数字面量
      ("1", 1),
      ("-2", -2),
      // 算术表达式
      ("1 + 2;", 3),
      ("1 - 2;", -1),
      ("1 * 2;", 2),
      ("3 / 2;", 1),
      ("1 + 2 * 3 - 3", 4),
      ("5 > 4 == 3 < 4;", 1),
      ("5 < 4 != 3 > 4;", 0),
      ("1 + (2 + 3) * 4;", 21),
      ("(5 + 5) * 2;", 20),
      ("-(5 + 5);", -10),
      ("10 + -5", 5),
      // if 表达式测试
      ("if 1 { } else { 1 }", 0),
      ("if 0 { 2 } else { 0 }", 0),
      ("if 0 { 2 }", 0),
      ("if 1 { 2 } else { 1 }", 2),
      ("if 0 { 2 } else { 1 }", 1),
      // while 表达式测试
      ("var i = 10; while i > 5 { i -= 1; }", 0),
      ("var i = 10; while i > 5 { i -= 1; }; i ", 5),
      // 函数测试
      // ("(fn test(n) { n })(3)", 3),
      // ("(fn fib(n) { if n <= 1 { 1 } else { fib(n - 1) + fib(n - 2) } })(5)", 8),
      ("fn test(n) { n } test(3);", 3),
      ("fn b(n) {n} fn test(n) { b(n) + b(n) } test(3);", 6),
    ]);
  }

  /// 评估程序 给出结果
  fn test_eval(t: Vec<(&str, isize)>) {
    for (input, expect) in t {
      let program = Paser::paser(input).unwrap();
      println!("{:}", &program.unparse());
      // println!("{:#?}", &program);

      let codes = Compiler::compile(&program).unwrap();

      // VM::print_codes(&codes);

      assert_eq!(VM::execute(&codes), expect);
    }
  }
}
