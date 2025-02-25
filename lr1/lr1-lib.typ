#let build(origin-gammer) = {
  /// 拓广文法
  let gammer = {
    let new_start = "S" + sym.prime
    let gammer = (
      nt: (new_start,) + origin-gammer.nt,
      t: origin-gammer.t,
      p: (:),
      s: new_start,
    )
    gammer.p.insert(new_start, (origin-gammer.s.codepoints(),))
    for (nt, rhss) in origin-gammer.p.pairs() {
      gammer.p.insert(nt, rhss.map(str.codepoints))
    }

    gammer
  }

  /// 获取 非终结符 + 终结符 的 first 集合
  let firsts = {
    let firsts = (:)

    // 终结符
    firsts.insert("#", ("#",))
    for t in gammer.t {
      firsts.insert(t, (t,))
    }

    // 非终结符
    let change = true
    while change {
      change = false

      for (nt, rhss) in gammer.p {
        let ntfirsts = firsts.at(nt, default: ())
        for rhs in rhss {
          // 起始终结符或者空字符串
          if rhs.len() < 1 or rhs.first() in gammer.t {
            let ch = if rhs.len() < 1 { sym.epsilon } else { rhs.first() }

            if not ntfirsts.contains(ch) {
              ntfirsts.push(ch)
              change = true
            }
          } else {
            // 终结符
            let next = true

            let index = 0
            while next and index < rhs.len() {
              next = false
              let ch = rhs.at(index)
              let fs = firsts.at(ch, default: ())

              for f in fs.filter(it => it != sym.epsilon and (it not in ntfirsts)) {
                ntfirsts.push(f)
                change = true
              }

              if sym.epsilon in fs {
                index += 1
                next = true
              }
            }
          }
        }
        firsts.insert(nt, ntfirsts)
      }
    }

    firsts
  }

  /// 获取 alpha 的 first 集
  let alpha-firsts(alpha) = {
    let afirsts = ()
    let next = true
    let index = 0
    while index < alpha.len() and next {
      next = false
      let fs = firsts.at(alpha.at(index))

      for f in fs.filter(it => it != sym.epsilon and (it not in afirsts)) {
        afirsts.push(f)
      }

      if fs.contains(sym.epsilon) {
        next = true
      }
      index += 1
    }

    if next { afirsts.push(sym.epsilon) }

    afirsts
  }

  ///
  let id2product(id) = {
    let (nt, i) = id.split("-")
    (nt, gammer.p.at(nt).at(int(i)))
  }

  ///
  let product2id(nt, index) = nt + "-" + str(index)

  /// 传入 leader，获取项目集闭包
  let closure(products) = {
    let change = true

    while change {
      change = false

      for (id, location, forward) in products {
        let (nt, rhs) = id2product(id)

        if location < rhs.len() and rhs.at(location) in gammer.nt {
          let next_nt = rhs.at(location)

          let new_forward = if location + 1 < rhs.len() {
            alpha-firsts(rhs.slice(location + 1))
          } else {
            forward
          }

          for (new_i, new_rhs) in gammer.p.at(next_nt).enumerate() {
            let new_id = product2id(next_nt, new_i)

            let index = 0
            while index < products.len() {
              let product = products.at(index)
              if new_id == product.at(0) and 0 == product.at(1) {
                break
              }
              index += 1
            }

            if index == products.len() {
              products.push((new_id, 0, new_forward))
              change = true
            } else {
              let old_forward = products.at(index).at(2)

              for f in new_forward.filter(x => x not in old_forward) {
                products.at(index).at(2).push(f)
                change = true
              }
            }
          }
        }
      }
    }

    products
  }

  /// 求项目集规范族与状态转化表
  let (items, dfa) = {
    let items = ()
    let dfa = (:)

    // I0
    items.push({
      let products = gammer.p.at(gammer.s).enumerate().map(((index, _)) => (product2id(gammer.s, index), 0, ("#",)))
      (leader: products.len(), p: closure(products))
    })

    // 求其他项目集
    let stack = (0,)
    while stack.len() > 0 {
      let from = stack.pop()
      let item = items.at(from)

      // 对项目集产生式分组
      let group = (:)
      for (index, product) in item.p.enumerate() {
        let (id, location, ..) = product
        let rhs = id2product(id).at(1)
        if location < rhs.len() {
          let next = rhs.at(location)
          let indices = group.at(next, default: ())
          indices.push(index)

          group.insert(next, indices)
        }
      }

      // 生成新的项目集
      for (next, indices) in group.pairs() {
        // leader 项目
        let products = indices.map(index => {
          let (id, location, forward) = item.p.at(index)
          (id, location + 1, forward)
        })

        let index = 0
        while index < items.len() {
          let item = items.at(index)
          // 只比较 leader 产生式
          if item.leader == products.len() and products == item.p.slice(0, item.leader) {
            break
          }
          index += 1
        }

        if index == items.len() {
          items.push((leader: products.len(), p: closure(products)))
          stack.push(index)
        }

        // DFA
        let (from, to) = (str(from), index)
        if dfa.at(from, default: none) == none {
          dfa.insert(from, (:))
        }
        dfa.at(from).insert(next, to)
      }
    }

    (items, dfa)
  }

  /// 符号与装填转换表中的位置映射
  let map = {
    let map = (:)

    for (index, t) in gammer.t.enumerate() {
      map.insert(t, index)
    }

    for (index, nt) in gammer.nt.filter(nt => gammer.s != nt).enumerate() {
      map.insert(nt, index + gammer.t.len() + 1)
    }
    map.insert("#", gammer.t.len())

    map
  }

  /// 转换表
  /// 0 接受，1 S，2 归约
  let analysis-table = {
    let analysis-table = range(0, items.len()).map(_ => range(0, map.len()).map(_ => ()))

    // 转换
    for (from, tos) in dfa.pairs() {
      for (x, to) in tos.pairs() {
        let index = map.at(x)
        analysis-table.at(int(from)).at(index).push(if index <= gammer.t.len() { (1, to) } else { to })
      }
    }

    // 归约
    for (index, value) in items.enumerate() {
      for (id, location, forward) in value.p {
        let (nt, rhs) = id2product(id)

        if rhs.len() == location {
          if gammer.s == nt {
            // 接受
            for x in forward {
              analysis-table.at(int(index)).at(map.at(x)).push(0)
            }
          } else {
            // 归约
            for x in forward {
              analysis-table.at(int(index)).at(map.at(x)).push((2, (id)))
            }
          }
        }
      }
    }

    analysis-table
  }

  /// LR1 分析函数
  /// @params s 待分析的文法
  let analysis(s) = {
    let s = (s + "#").codepoints()

    let steps = ()
    let syntex-tree = ()

    let state-stack = () // 状态栈
    let symbol-stack = () // 符号栈

    state-stack.push(0)
    let index = 0

    while index < s.len() {
      let step = (state-stack, symbol-stack, index, none, none)

      let char = s.at(index)

      if map.at(char, default: none) == none {
        step.at(3) = "FAILED（词法错误）"

        steps.push(step)
        break
      } else {
        let action = analysis-table.at(state-stack.last()).at(map.at(char))

        if action.len() < 1 {
          step.at(3) = "FAILED"

          steps.push(step)
          break
        } else {
          let (action,) = action

          if action == 0 {
            step.at(3) = (
              if index == s.len() - 1 and symbol-stack.len() == 1 and symbol-stack.first() == origin-gammer.s {
                "ACC"
              } else {
                "FAILED"
              }
            )

            steps.push(step)
            break
          } else if action.at(0) == 1 {
            // 移进
            state-stack.push(action.at(1))
            symbol-stack.push(char)
            step.at(3) = action
            steps.push(step)

            syntex-tree.push(char)
          } else if action.at(0) == 2 {
            // 归约
            let (nt, rhs) = id2product(action.at(1))

            for i in range(0, rhs.len()) {
              let _ = state-stack.pop()
              let _ = symbol-stack.pop()
            }

            let goto = analysis-table.at(state-stack.last()).at(map.at(nt)).at(0)

            state-stack.push(goto)
            symbol-stack.push(nt)

            step.at(3) = action
            step.at(4) = goto
            steps.push(step)

            let symbols = ()
            for i in range(0, rhs.len()) {
              symbols.push(syntex-tree.pop())
            }
            syntex-tree.push((nt, ..symbols.rev()))

            continue
          }
        }
      }

      index += 1
    }

    (steps, if syntex-tree.len() > 1 { none } else { syntex-tree.last() })
  }

  /// 验证该文法是否是LR(1)
  let valid = analysis-table.fold(true, (prev, row) => row.fold(true, (prev, col) => prev and col.len() < 2) and prev)

  (
    firsts: firsts, // 终结符与非终结符的FIRST集
    valid: valid, // 是否为LR1文法
    gammer: gammer, // 拓广文法
    items: items, // 项目集规范族
    id2product: id2product,
    transition-table: dfa, // 状态转换表
    map: map,
    analysis-table: analysis-table, // 预测分析表
    analysis: if valid { analysis } else { none }, // 分析程序
  )
}
