#let 字号 = (
  初号: 42pt, 小初: 36pt, 一号: 26pt, 小一: 24pt, 二号: 22pt, 小二: 18pt, 三号: 16pt,
  小三: 15pt, 四号: 14pt, 中四: 13pt, 小四: 12pt, 五号: 10.5pt, 小五: 9pt, 六号: 7.5pt,
  小六: 6.5pt, 七号: 5.5pt, 小七: 5pt,
)

#let 字体 = (
  仿宋: ("Times New Roman", "FangSong"),
  宋体: ("Times New Roman", "SimSun"),
  黑体: ("Times New Roman", "SimHei"),
  楷体: ("Times New Roman", "KaiTi"),
  数学: ("New Computer Modern Math", "SimSun"),
  代码: ("Fira Code", "SimSun"),
)

#let chineseheading(to: none, it) = {
  set par(first-line-indent: 0em)
  set text(font: 字体.黑体)
  set block(above: 1.5em, below: 1.5em)

  let sizedheading(it, size) = {
    set text(size, font:字体.黑体)
    if it.numbering != none {
      counter(heading).display()
    }
    h(0.5em)
    it.body
  }

  if it.level == 1 {            // 章
    sizedheading(it, 字号.小二)
  } else if it.level == 2 {     // 节
    sizedheading(it, 字号.小三)
  } else if it.level == 3 {     // 条
    sizedheading(it, 字号.四号)
  } else if it.level == 4 {     // 款
    sizedheading(it, 字号.小四)
  } else {
    sizedheading(it, 字号.小四)
  }
}
