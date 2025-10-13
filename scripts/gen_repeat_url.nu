use ./helpers.nu [human_count]

# Scenario 1: Duplicate some entries in the existing URL list by a percentage (to create duplicates)
def main [
  --pct: int = 25,                                       # Required: percentage of duplicates to add, between 0..100, e.g. 25
  --input_path  (-i): path = "./scripts/data/urls.txt",
  --output_path (-o): path = "./tests/perf/data/",
  --field       (-f): string = "url"                     # Reserved parameter (same as original), not used here
  --limit       (-l): int = 100_000                      # Max number of lines to read, default: 100K
  --start       (-s): int = 0                            # New: Start from this line (0-based)
] {
  # Validate parameters
  if $pct < 0 or $pct > 100 {
    error make { msg: "pct must be between 0 and 100" }
  }
  if $limit <= 0 {
    error make { msg: "limit must be greater than 0" }
  }

  # Read the original list, trim whitespace, skip empty lines, and apply line limits (✂️)
  let orig_list = (
    open $input_path
    | lines
    | skip $start
    | take $limit
    | each {|x| $x | str trim }
    | where {|x| $x != "" }
  )

  let orig_count = ($orig_list | length)
  if $orig_count == 0 {
    error make { msg: $"Input file is empty or no data read from ($input_path)" }
  }

  # Calculate the number of duplicates to add
  let pct_decimal = ($pct | into float | $in / 100)
  let add_count = (if $pct_decimal == 1 {
      error make { msg: "pct cannot be 100 — infinite duplicates would be required 😅" }
  } else {
      ($orig_count * $pct_decimal) / (1 - $pct_decimal)
  } | math round)

  let total_count = $orig_count + $add_count

  # Sample and shuffle
  let dup_list = (if $add_count > 0 { $orig_list | shuffle | take $add_count } else { [] })
  let mixed = ($orig_list | append $dup_list) | shuffle

  # Save output file
  mkdir $output_path
  let size_tag = (human_count $orig_count)
  let name = $"url_repeat_($pct)%_($size_tag).txt"
  $mixed | save -f ($output_path | path join ($name))

  # Print summary
  let eff_pct = (if $total_count == 0 { 0 } else { ($add_count * 100) // $total_count })
  print $"✅ Generation completed: ($output_path | path join ($name))"
  print $"Lines read: ($orig_count), Original count: ($orig_count), Duplicates added: ($add_count), Total: ($total_count), Duplicate ratio: ($eff_pct)%"
}
