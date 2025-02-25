#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NameTableKind {
  Constant,  // 常量
  Variable,  // 变量
  Proceduce, // 过程
}

#[derive(Debug)]
pub struct NameTableItem {
  pub name: String,
  pub kind: NameTableKind,
  pub value: isize,
  pub level: usize,
  pub addr: isize, // 语句代码地址
  pub size: usize,
}

/// 符号表
#[derive(Debug)]
pub struct NameTable {
  pub items: Vec<NameTableItem>,
  tx: usize, // table pointer 指向最后一项
}

impl NameTable {
  pub fn new() -> Self {
    let main_proc = NameTableItem {
      name: "_main_".to_string(),
      kind: NameTableKind::Proceduce,
      value: 0, // const: ; function: 起始地址
      level: 0, // const 用不到
      addr: 0,  // var: 相对基地址.
      size: 0,  // procedce, 需要分配的大小
    };

    NameTable { items: vec![main_proc], tx: 0 }
  }

  /// 添加到符号表中
  pub fn add(&mut self, item: NameTableItem) {
    self.tx += 1;

    if self.tx < self.items.len() {
      self.items[self.tx] = item;
    } else {
      self.items.push(item);
    }
  }

  /// 添加 常量
  pub fn add_const(&mut self, name: &str, level: usize, value: isize) {
    self.add(NameTableItem { name: name.to_string(), kind: NameTableKind::Constant, value, level, addr: 0, size: 0 })
  }

  ///
  /// 添加变量
  ///
  pub fn add_variable(&mut self, name: &str, level: usize, raddr: isize) {
    self.add(NameTableItem {
      name: name.to_string(),
      kind: NameTableKind::Variable,
      value: 0,
      level,
      addr: raddr,
      size: 0,
    })
  }

  ///  添加 过程
  pub fn add_proceduce(&mut self, name: &str, level: usize) {
    self.add(NameTableItem {
      name: name.to_string(),
      kind: NameTableKind::Proceduce,
      value: 0,
      level,
      addr: 0,
      size: 0,
    })
  }

  /// 回退多少 下标
  pub fn rollback(&mut self, a: usize) {
    self.tx -= a;
  }

  pub fn tx(&self) -> usize {
    self.tx
  }

  pub fn find(&self, ident: &str) -> Option<&NameTableItem> {
    self.items.iter().take(self.tx() + 1).rev().find(|&cur| cur.name == ident)
  }
  ///  从后向前找
  pub fn find_kind(&self, ident: &str, kind: NameTableKind) -> Option<&NameTableItem> {
    match self.find(ident) {
      Some(item) if item.kind == kind => Some(item),

      Some(_) => panic!("expect {:?}", kind),
      None => panic!("expect ident"),
    }
  }

  // /// 返回在符号表中的位置
  // pub fn position(&self, ident: &str) -> Option<usize> {
  //   self.items.iter().rev().position(|cur| cur.name == ident)
  // }

  #[allow(unused)]
  pub fn print_nametable(&self) {
    for (index, item) in self.items.iter().enumerate().take(self.tx() + 1) {
      println!("{:}, {:?}", index, item);
    }

    println!("-----------------------");
  }
}
