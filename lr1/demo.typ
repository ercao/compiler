#import "@preview/cetz:0.1.2"
#import "lib/chinese.typ": 字体, 字号, chineseheading

#import "./lr1-lib.typ": build
#import "./test.typ": dataset

#let LR1 = "LR(1)"

#show: it => {
  set text(font: 字体.宋体, cjk-latin-spacing: auto, lang: "zh")
  set heading(
    numbering: (..nums) => {
      if nums.pos().len() == 1 {
        numbering("一", ..nums)
      } else {
        numbering("1.1", ..nums)
      }
    },
  )
  show heading: chineseheading
  show math.equation: set text(font: 字体.数学)
  show raw: set text(font: 字体.代码)
  show figure: set block(breakable: true)
  set table(stroke: 0.5pt)
  set page(paper: "a4", numbering: "1")
  show figure.caption: set text(size: 字号.小五)
  show figure.where(kind: "table"): set figure.caption(position: top)

  it
}

#let to-math(s) = s.codepoints().map(ch => $#ch$).join()

#let product(lhs, rhs) = {
  let to-math(s) = s.map(ch => $#ch$).join()

  $#to-math(lhs.codepoints())->#if rhs.len() < 1 { sym.epsilon } else { to-math(rhs) }$
}

#let products(lhs, rhss) = {
  let rhss = rhss.map(rhs => { if rhs.len() < 1 { (sym.epsilon,) } else { rhs } })

  let to-math(s) = s.map(ch => $#ch$).join()

  $#to-math(lhs.codepoints())->#rhss.map(to-math).join("|")$
}

#(
  dataset
    .map(((title, gammer, tests)) => [
      #let lr1 = build(gammer)

      = #title

      == 文法

      #table(
        columns: 3,
        column-gutter: 2em,
        stroke: 0pt,
        align: (top, horizon, top) + (left, left, left),
        $\
      G[#gammer.s]: #gammer.p.pairs().map(((nt, rhss)) => {
        $& #products(nt,  rhss.map(str.codepoints))$
      }).join(linebreak())$,
        $=>^"拓广文法"$,
        $\
      G[#lr1.gammer.s]: #lr1.gammer.p.pairs().map(((nt, rhss)) => $& #products(nt,  rhss)$).join(linebreak())$,
      )

      == 非终结符FIRST集

      $\
    #gammer.nt.map(((nt) => { $ "FIRST"(#nt) = { #to-math(lr1.firsts.at(nt).join(",")) } $ })).join(linebreak())$

      == 项目集规范族与预测分析表

      #[
        #set text(size: 字号.五号)

        #figure(
          caption: [#LR1 项目集表],
          kind: table,
          table(
            columns: 3,
            align: top + left,
            stroke: 0.5pt,

            ..lr1
              .items
              .enumerate()
              .map(((index, item)) => {
                let (leader, p) = item

                $\
            S_#index : #p.enumerate().map(((index, item)) => {
              let (id, location, forward) = item
              let p = (lr1.id2product)(id)
              let rhs = p.at(1)
              rhs.insert(location, sym.dot)

              set text(orange) if index < leader

              $& #product(p.at(0), rhs) &#h(0.5em), #forward.join()$
            }).join(linebreak())$
              })
              .flatten()
          ),
        )

        #figure(
          caption: [#LR1 预测分析表],
          kind: table,
          table(
            columns: 1 + gammer.t.len() + 1 + gammer.nt.len(),
            align: center + horizon,
            stroke: 0.5pt,

            table.header(
              repeat: true,

              table.cell(rowspan: 2)[状态],
              table.cell(colspan: gammer.t.len() + 1, to-math("ACTION")),
              table.cell(colspan: gammer.nt.len(), to-math("GOTO")),

              ..gammer.t.map(math.equation),
              to-math("#"),
              ..gammer.nt.map(math.equation),
            ),

            ..lr1
              .analysis-table
              .enumerate()
              .map(((index, row)) => {
                ([#index],)

                row
                  .enumerate()
                  .map(((index, col)) => {
                    set text(red) if col.len() > 1
                    col
                      .map(action => {
                        if index <= gammer.t.len() {
                          if action == none { } else if action == 0 { to-math("ACC") } else if action.at(0) == 1 {
                            $S_#to-math(str(action.at(1)))$
                          } else if action.at(0) == 2 { $R_#to-math(action.at(1))$ }
                        } else {
                          to-math(str(action))
                        }
                      })
                      .join(",")
                  })
              })
              .flatten(),
          ),
        )
      ]

      #if lr1.valid [
        #if tests.len() < 1 { return }

        == 分析过程

        #[
          #set text(size: 字号.五号)
          #(
            tests
              .map(s => {
                let s = tests.at(0)

                let (steps, syntex-tree) = (lr1.analysis)(s)

                figure(
                  caption: [#to-math(s) 的 #LR1 分析过程],
                  kind: table,

                  table(
                    columns: 6,
                    stroke: 0.5pt,
                    align: center + horizon,

                    table.header(
                      [步骤],
                      [状态栈],
                      [符号栈],
                      [输入串],
                      to-math("ACTION"),
                      to-math("GOTO"),
                    ),

                    ..steps
                      .enumerate()
                      .map(((index, row)) => {
                        (
                          str(index + 1),
                          to-math(row.at(0).map(str).join(",")),
                          to-math("#" + row.at(1).join()),
                          to-math(s.codepoints().slice(row.at(2)).join() + "#"),
                          {
                            let action = row.at(3)
                            if type(action) == str {
                              to-math(action)
                            } else if action.at(0) == 1 {
                              $S_#to-math(str(action.at(1)))$
                            } else if action.at(0) == 2 {
                              $R_#to-math(action.at(1))$
                            }
                          },
                          [#row.at(4)],
                        )
                      })
                      .flatten()
                  ),
                )

                if syntex-tree != none {
                  figure(
                    caption: [#s 的 #LR1 语法树],
                    cetz.canvas({
                      import cetz.draw: *

                      cetz.tree.tree(
                        syntex-tree,
                        content: (padding: .1),
                        line: (stroke: 0.5pt),
                        draw-node: (node, _) => {
                          content((), math.equation(node.content))
                        },
                      )
                    }),
                  )
                }
              })
              .join()
          )
        ]
      ] else {
        text(red)[!!! 该文法非LR1文法]
      }
    ])
    .join()
)
