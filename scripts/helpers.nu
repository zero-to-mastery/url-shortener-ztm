# Convert numbers to short labels: 999 -> "999", 10_000 -> "10K", 2_300_000 -> "2.3M", 4_100_000_000 -> "4.1B"
export def human_count [n: int] {
  if $n < 1000 {
    $n | into string
  } else if $n < 1_000_000 {
    ((($n | into float) / 1000) | math round | into int | into string) + "K"
  } else if $n < 1_000_000_000 {
    ((($n | into float) / 1_000_000) | math round | into int | into string) + "M"
  } else {
    ((($n | into float) / 1_000_000_000) | math round | into int | into string) + "B"
  }
}

# Generate a random custom short ID with given length and charset
export def gen_custom_id [length: int, charset: string] {
  let chars = $charset | split chars
  let n = ($chars | length)
  (1..$length | each { $chars | get (random int ..<$n) }) | str join
}

# Generate a timestamp suffix, e.g. _20251011123045
export def timestamp [] { date now | format date "%Y%m%d%H%M%S" }


export def exists_and_nonempty [p : path]: nothing -> bool {
  try { (ls ($p | into glob) | get size.0 | into int) > 0 } catch { false }
}

export def cal_gen_pct [num: int pct: int]: nothing -> int {
    let pct_decimal = ($pct | into float | $in / 100)
  let counts = (if $pct_decimal == 1 {
      error make { msg: "pct cannot be 100" }
  } else {
      ($num * $pct_decimal) / (1 - $pct_decimal)
  } | math round)

  $counts
}