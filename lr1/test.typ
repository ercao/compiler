#let dataset = (
  // (
  //   "LR（1） 1",
  //   (
  //     nt: ("S", "A"),
  //     t: ("a", "b", "c", "d", "e"),
  //     p: (
  //       S: ("aAd", "bAc", "aec", "bed",),
  //       A: ("e", ""),
  //     ),
  //     s: "S"
  //   ),
  //   (),
  // ),

  // (
  //   "LR（1） 2",
  //   (
  //     nt: ("E", "A", "B"),
  //     t: ("a", "b", "c", "d"),
  //     p: (
  //       E: ("aA", "bB"),
  //       A: ("cA", "d"),
  //       B: ("cB", "d"),
  //     ),
  //     s: "E"
  //   ),
  //   ("bcc", )
  // ),

  // (
  //   "LR（1） 3",
  //   (
  //     nt: ("S", "A", "B"),
  //     t: ("p", "q", "a", "c", "b", "d"),
  //     p: (
  //       S: ("Ap","Bq"),
  //       A: ("a","cA"),
  //       B: ("b","dB"),
  //     ),
  //     s: "S"
  //   ),
  //   ()
  // ),

  (
    "四则运算文法",
    (
      nt:("E", "T", "F"),
      t: (sym.plus, sym.minus, sym.times, sym.div, "(", ")", "n"),
      p: (
        E: ("E"+sym.plus+"T", "E"+sym.minus+"T", "T"),
        T: ("T"+sym.times+"F", "T"+sym.div+"F", "F"),
        F: ("(E)", "n"),
      ),
      s: "E"
    ),
    ("(n"+sym.plus+"n)"+sym.times+"n"+sym.minus+"n"+sym.div+"n", )
  ),

  (
    "非LR1文法",
    (
      nt: ("S", "A", "B"),
      t: ("a", "b"),
      p: (
        S: ("Aa","Bb"),
        A: ("Ab", "","b"),
        B: ("Ba", "")
      ),
      s: "S"
    ),
    (),
  ),
)
