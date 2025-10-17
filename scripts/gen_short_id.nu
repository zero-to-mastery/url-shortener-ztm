use ./helpers.nu [gen_custom_id human_count timestamp]

def main [
  --pct: int = 25,                                                # % of new IDs to generate (0–100)
  --input_path  (-i): path = "./scripts/data/urls.csv",           # Input CSV path (must have 'id')
  --config_path (-c): path = "./configuration/generator.yml",     # Generator config (length, alphabet)
  --output_path (-o): path = "./tests/perf/data/"                 # Output folder
] {
  # Validate parameters
  if $pct < 0 or $pct > 100 {
    error make { msg: "pct must be between 0 and 100" }
  }

  # Load config & original data
  let cfg = open $config_path
  let orig_list = (open $input_path | get code)
  let orig_count = ($orig_list | length)
  if $orig_count == 0 {
    error make { msg: $"No 'id' field found or file is empty: ($input_path)" }
  }

  # Calculate number to generate
  let pct_decimal = ($pct | into float | $in / 100)
  let add_count = (if $pct_decimal == 1 {
      error make { msg: "pct cannot be 100" }
  } else {
      ($orig_count * $pct_decimal) / (1 - $pct_decimal)
  } | math round)

  let total_count = $orig_count + $add_count

  # Generate IDs
  let indices = (if $add_count > 0 { 1..$add_count } else { [] })
  let gen_list = (
    $indices
    | each { gen_custom_id $cfg.shortener.length $cfg.shortener.alphabet }
  )

  # Merge & shuffle
  let combined = ($orig_list | append $gen_list) | shuffle

  # Save output
  mkdir $output_path
  let size_tag = (human_count $orig_count)
  let name = $"db_code_extra_($pct)%_($size_tag)_(timestamp)"
  $combined | save -f ($output_path | path join ($name + ".txt"))

  # Summary
  let eff_pct = (if $total_count == 0 { 0 } else { ($add_count * 100) // $total_count })
  print $"✅ Generated: ($output_path | path join ($name + '.txt'))"
  print $"Original: ($orig_count), New: ($add_count), Total: ($total_count), Ratio: ($eff_pct)%"
}
