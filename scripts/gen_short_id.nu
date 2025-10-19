# scripts/gen_short_id.nu
use ./helpers.nu [gen_custom_id human_count timestamp]

def main [
  --pct: int = 25,                                                # % of new IDs to generate (0–100)
  --input_path  (-i): path = "./scripts/data/urls.csv",           # Input CSV path (must have column 'code')
  --config_path (-c): path = "./configuration/generator.yml",     # Generator config (length, alphabet)
  --output_path (-o): path = "./tests/perf/data/"                 # Output folder
] {
  if $pct < 0 or $pct > 100 {
    error make { msg: "pct must be between 0 and 100" }
  }

  let cfg = open $config_path
  let orig_list = (open $input_path | get code)
  let orig_count = ($orig_list | length)
  if $orig_count == 0 {
    error make { msg: $"No 'code' field found or file is empty: ($input_path)" }
  }

  let pct_decimal = ($pct | into float | $in / 100)
  let add_count = (if $pct_decimal == 1 {
      error make { msg: "pct cannot be 100" }
  } else {
      ($orig_count * $pct_decimal) / (1 - $pct_decimal)
  } | math round | into int)

  let indices = (if $add_count > 0 { 1..$add_count } else { [] })
  let gen_list = ($indices | each { gen_custom_id $cfg.shortener.length $cfg.shortener.alphabet })

  let combined = ($orig_list | append $gen_list) | shuffle

  mkdir $output_path | ignore
  let size_tag = (human_count $orig_count)
  let name = $"db_code_extra_($pct)%_($size_tag)_(timestamp)"
  $combined | save -f ($output_path | path join ($name + ".txt"))

  let total_count = $orig_count + $add_count
  let eff_pct = (if $total_count == 0 { 0 } else { (100 * $add_count / $total_count | into int) })
  print $"✅ Generated: ($output_path | path join ($name + '.txt'))"
  print $"Original: ($orig_count), New: ($add_count), Total: ($total_count), Ratio: ($eff_pct)%"
}
