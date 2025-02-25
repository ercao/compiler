# LR1 语法分析器

## 开发工具

`typst 0.12`

## 功能

- [x] 根据 #LR1 分析法编写一个语法分析程序，可选择以下一项作为分析算法的输入：

  - [ ] 直接输入根据已知文法构造的 LR(1) 分析表；
  - [ ] 输入已知文法的项目集规范族和转换函数，由程序自动生成 LR(1) 分析表；
  - [x] 输入已知文法，由程序自动构造识别该文法活前缀 DFA 并生成 LR(1) 分析表。

- [x] 所开发的程序可适用于不同的文法和任意输入串，且能判断该文法是否为 #LR1 文法。
- [x] 对输入的任意符号串，所编制的语法分析程序应能正确判断此串是否为文法的句子（句型分析），并要求输出分析过程与该符号串的语法树。

## 使用

```typst
#import "./lr1-lib.typ": build

// 定义LR1文法四元组
// 四则运算表达式
#let gammer = (
  nt:("E", "T", "F"),
  t: (sym.plus, sym.minus, sym.times, sym.div, "(", ")", "n"),
  p: (
    E: ("E"+sym.plus+"T", "E"+sym.minus+"T", "T"),
    T: ("T"+sym.times+"F", "T"+sym.div+"F", "F"),
    F: ("(E)", "n"),
  ),
  s: "E"
)

// 构建LR1分析器
#let lr1 = build(gammer)

// 终结符与非终结符的 FIRST 集
#lr1.firsts

// 是否为有效的lr1文法
#lr1.valid

// 拓广文法
#lr1.gammer

// 项目集规范族
#lr1.items

// 预测分析表
#lr1.analysis-table

// 句型分析
#let s = "(n"+sym.plus+"n)"+sym.times+"n"+sym.minus+"n"+sym.div+"n"
#lr1.analysis(s)
```

具体实例请查看 `demo.typ` 文件
