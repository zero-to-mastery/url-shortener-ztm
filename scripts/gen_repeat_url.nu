use ./helpers.nu [human_count]

# Scenario 1: Duplicate entries in an existing URL list by a percentage to introduce duplicates.
def main [
  --pct: int = 25,                               # Percentage of duplicates to add (0..100, 25 = add 25% more lines)
  --input_path  (-i): path = "./scripts/data/urls.txt",
  --output_path (-o): path = "./tests/perf/data/",
  --field       (-f): string = "url",            # Reserved alignment with other generators (not used here)
  --limit       (-l): int = 100_000,             # Max number of lines to read
  --offset      (-s): int = 0                    # Start from this 0-based line
] {
  # Validate parameters
  if $pct < 0 or $pct > 100 {
    error make { msg: "pct must be between 0 and 100" }
  }
  if $limit <= 0 {
    error make { msg: "limit must be greater than 0" }
  }

  # Read the original list, trim whitespace, skip empties, and apply window
  let orig_list = (
    open $input_path
    | lines
    | skip $offset
    | take $limit
    | each {|x| $x | str trim }
    | where {|x| $x != "" }
  )

  let orig_count = ($orig_list | length)
  if $orig_count == 0 {
    error make { msg: $"Input file is empty or no data read from ($input_path)" }
  }

  # Compute how many duplicates to add to reach a final duplicate ratio of `pct`
  let pct_decimal = ($pct | into float | $in / 100)
  let add_count = (if $pct_decimal == 1 {
      error make { msg: "pct cannot be 100 — infinite duplicates would be required" }
  } else {
      ($orig_count * $pct_decimal) / (1 - $pct_decimal)
  } | math round | into int)

  let total_count = $orig_count + $add_count

  # Sample duplicates and shuffle with originals
  let dup_list = (if $add_count > 0 { $orig_list | shuffle | take $add_count } else { [] })
  let mixed = ($orig_list | append $dup_list) | shuffle

  # Save output file
  mkdir $output_path | ignore
  let size_tag = (human_count $orig_count)
  let name = $"url_repeat_($pct)%_($size_tag).txt"
  $mixed | save -f ($output_path | path join ($name))

  # Summary
  let eff_pct = (if $total_count == 0 { 0 } else { (100 * $add_count / $total_count | into int) })
  print $"✅ Generation completed: ($output_path | path join ($name))"
  print $"Lines read: ($orig_count), Duplicates added: ($add_count), Total: ($total_count), Duplicate ratio: ($eff_pct)%"
}
