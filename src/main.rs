use std::{
  fs::File,
  io::{Error, Read},
};

use ariadne::{Label, Report, ReportKind, Source};
use pl0::{compiler::Compiler, parser::Paser, vm::VM, SpanOffset};

/// 打印错误
fn print_errors(title: &str, errors: &[(String, SpanOffset)], input: &str) {
  Report::build(ReportKind::Error, (), 0)
    .with_labels(
      errors.iter().map(|(err, pos)| Label::new(pos.begin..pos.end).with_message(err).with_color(ariadne::Color::Red)),
    )
    .with_message(title)
    .finish()
    .print(Source::from(&input))
    .unwrap();
}

fn main() -> Result<(), Error> {
  let input = "examples/fib.pl0";

  let mut file = File::open(input)?;
  let mut input = String::new();
  file.read_to_string(&mut input)?;

  let program = match Paser::paser(&input) {
    Ok(program) => program,
    Err(err) => {
      print_errors("语法解析错误", &[err], &input);

      return Ok(());
    }
  };

  println!("{:^-20}", "抽象语法树");
  println!("{:?}", &program);

  let codes = match Compiler::compile(&program) {
    Ok(program) => program,
    Err(errors) => {
      print_errors("编译错误", &errors, &input);

      return Ok(());
    }
  };

  println!("{:^-20}", "目标代码");
  VM::print_codes(&codes);

  println!("{:^-20}", "程序执行结果");
  VM::execute(&codes);

  Ok(())
}
