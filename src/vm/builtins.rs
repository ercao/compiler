use std::{
  collections::HashMap,
  io::{self, Write},
};

/// 内建函数
pub struct Builtins {
  map: HashMap<String, usize>,
  arr: Vec<(fn(Vec<isize>) -> isize, String)>,
}

impl Builtins {
  pub fn new() -> Self {
    let arr: Vec<(fn(Vec<isize>) -> isize, String)> = vec![
      (Self::helloworld, "helloworld".to_string()),
      (Self::print, "print".to_string()),
      (Self::println, "println".to_string()),
    ];
    let mut map = HashMap::new();

    arr.iter().enumerate().for_each(|(index, (_, name))| {
      map.insert(name.to_string(), index);
    });

    Builtins { map, arr }
  }

  /// 查询内建函数
  pub fn lookup(&self, name: &str) -> Option<usize> {
    self.map.get(name).map(|&x| x)
  }

  /// 调用函数
  pub fn call(&self, id: usize, args: Vec<isize>) -> isize {
    self.arr[id].0(args)
  }

  fn helloworld(_args: Vec<isize>) -> isize {
    _args.iter().for_each(|arg| {
      println!("{:?}", arg);
    });
    println!("hello world");
    0
  }

  fn print(args: Vec<isize>) -> isize {
    print!("{:?}", args.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "));
    io::stdout().flush().unwrap();
    0
  }

  fn println(args: Vec<isize>) -> isize {
    Self::print(args);
    println!();
    0
  }
}
