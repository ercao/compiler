pub mod builtins;

use std::fmt::Debug;

use self::builtins::Builtins;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
  None, // 空指令

  Lit(isize),            // 将指定 字面量压入栈中
  Lod(usize, isize),     // 将指定地址的压入栈顶
  Lod1(usize),           // 将栈中 相对栈顶 offset 位置的数压入栈顶
  Sto(usize, isize),     // 将栈顶元素放入指定地址
  Int(usize),            // 分配内存
  Jmp(usize),            // 无条件跳转
  Jpc(usize),            // 栈顶为 0 时跳转
  Cal(usize),            // 调用函数, 地址为栈顶的值
  Builtin(usize, usize), // 调用内建函数
  Ret,                   // 将栈顶元素返回
  CallClean(usize),      // 清理调用 函数后 上级函数的垃圾数据
  EnterScope,            // 进入作用域
  LeaveScope,            // 离开作用域

  // 一元操作
  Not, // ! 逻辑取反

  // 二元操作
  Add, // 加法
  Sub, // 减法
  Div, // 触发
  Mul, // 乘法
  Eq,  // ==
  Ne,  // !=
  Lt,  // <
  Le,  // <=
  Gt,  // >
  Ge,  // >=
}

pub struct VM {
  builtins: Builtins,
  stack: Vec<isize>, // 栈
  ip: usize,         // 指令指针
  bp: usize,         // 基地址指针
  sp: usize,         // 栈顶
}

impl VM {
  pub fn new() -> Self {
    VM {
      builtins: Builtins::new(),
      stack: vec![],
      ip: 0, // 下一条执行命令的位置
      bp: 0,
      sp: 0, // 指向可以使用的位置
    }
  }

  #[allow(unused)]
  pub fn print_codes(codes: &[Opcode]) {
    codes.iter().enumerate().for_each(|(index, code)| {
      println!("{:-04X}H {:?}", index, code);
    })
  }

  /// 执行虚拟机指令
  ///
  /// codes: 虚拟机指令集
  pub fn execute(codes: &[Opcode]) -> isize {
    if codes.is_empty() {
      return 0;
    }

    let mut vm = VM::new();

    // Self::print_codes(codes);

    loop {
      let instruction = codes[vm.ip];
      // println!("当前指令: {:?}, {:?}", vm.pc, &instruction,);

      vm.ip += 1;

      match instruction {
        Opcode::CallClean(num) => {
          let result = vm.pop();
          vm.sp -= num;
          vm.push(result);
        }
        Opcode::Sto(rlevel, address) => {
          // 将栈顶与 指定位置
          let address = (vm.base(rlevel) as isize + address) as usize;
          vm.stack[address] = vm.stack[vm.sp - 1];
        }

        Opcode::Cal(rlevel) => {
          let ip = vm.pop() as usize;

          vm.reserve(3);
          vm.stack[vm.sp] = vm.base(rlevel) as isize;
          vm.stack[vm.sp + 1] = vm.bp as isize;
          vm.stack[vm.sp + 2] = vm.ip as isize;
          vm.bp = vm.sp;
          vm.ip = ip;
        }

        Opcode::Ret => {
          let x = vm.pop();
          vm.sp = vm.bp;
          vm.bp = vm.stack[vm.sp + 1] as usize;
          vm.ip = vm.stack[vm.sp + 2] as usize;

          vm.push(x);
        }

        Opcode::EnterScope => {
          vm.reserve(1);
          vm.stack[vm.sp] = vm.bp as isize; // 记录上一层作用域的基地止
          vm.bp = vm.sp;
        }

        Opcode::LeaveScope => {
          let x = vm.pop();
          vm.sp = vm.bp;
          vm.bp = vm.stack[vm.bp] as usize;
          vm.push(x);
        }

        Opcode::Builtin(id, argc) => {
          // 调用内建函数
          let args = (0..argc).map(|_| vm.pop()).collect();
          let result = vm.builtins.call(id, args);
          vm.push(result);
        }

        Opcode::Int(num) => {
          vm.reserve(num);
          vm.sp += num;
        }

        Opcode::Jmp(address) => vm.ip = address, //
        //
        Opcode::Jpc(address) => {
          if vm.pop() == 0 {
            vm.ip = address;
          }
        }

        Opcode::Lit(value) => vm.push(value),
        Opcode::Lod(rlevel, address) => {
          let address = (address + vm.base(rlevel) as isize) as usize;
          vm.push(vm.stack[address]);
        }

        Opcode::Lod1(offset) => {
          let address = (vm.sp - offset) as usize;
          vm.push(vm.stack[address]);
        }

        Opcode::Not => {
          let operator = !(vm.pop() != 0) as isize;
          vm.push(operator);
        }

        _ => {
          let op2 = vm.pop();
          let op1 = vm.pop();

          match instruction {
            Opcode::Add => vm.push(op1 + op2),
            Opcode::Sub => vm.push(op1 - op2),
            Opcode::Div => vm.push(op1 / op2),
            Opcode::Mul => vm.push(op1 * op2),
            Opcode::Lt => vm.push((op1 < op2) as isize),
            Opcode::Gt => vm.push((op1 > op2) as isize),
            Opcode::Le => vm.push((op1 <= op2) as isize),
            Opcode::Ge => vm.push((op1 >= op2) as isize),
            Opcode::Eq => vm.push((op1 == op2) as isize),
            Opcode::Ne => vm.push((op1 != op2) as isize),
            _ => unreachable!(),
          };
        }
      }

      // println!(
      //   "{:?}, sp: {:?}, ip: {:?}, bp: {:?}",
      //   &vm.stack[0..vm.sp],
      //   vm.sp,
      //   vm.ip,
      //   vm.bp
      // );

      if vm.ip == 0 {
        break;
      }
    }

    if vm.sp != 1 {
      panic!("虚拟机栈未清理干净");
    }
    vm.stack[0]
  }

  /// 压栈
  fn push(&mut self, value: isize) {
    self.reserve(1);
    self.stack[self.sp] = value;
    self.sp += 1;
  }

  /// 弹栈
  fn pop(&mut self) -> isize {
    self.sp -= 1;
    self.stack[self.sp]
  }

  /// 通过过程基址求上 level 层过程的基地止
  fn base(&self, rlevel: usize) -> usize {
    (0..rlevel).fold(self.bp, |pre, _| self.stack[pre] as usize)
  }

  ///
  fn reserve(&mut self, additional: usize) {
    let expect_len = self.sp + additional;

    if self.stack.len() < expect_len {
      (0..expect_len - self.stack.len()).for_each(|_| self.stack.push(0));
    }
  }
}

/// 运行时错误
pub enum RuntimeError {
  IllegalFunctionAddress(usize), // ()
}

#[cfg(test)]
mod tests {
  #[test]
  fn test() {}
}
