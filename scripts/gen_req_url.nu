use ./helpers.nu [gen_custom_id]

const alpha = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_-"

export def gen_redirect_reqs [
   --code_path:    path = ./scripts/data/in_db/primary_codes_no_alias.csv
   --alias_path:   path = ./scripts/data/in_db/aliases.csv
   --code_offset:  int = 0        # Line offset to start read code
   --alias_offset: int = 0        # Line offset to start read code
   --limit:        int = 100_000  # Total number of redirect requests to produce
   --code_pct:     int = 75       # Percentage of requests using primary codes
   --alias_pct:    int = 15       # Percentage of requests using aliases
   --fake_pct:     int = 10       # Percentage of fake/random requests
]: nothing -> list<list<string>> {

   # --- Validate percentages: each in [0,100] and sum == 100 -----------------
   if (($code_pct   < 0) or ($code_pct  > 100) or
       ($alias_pct  < 0) or ($alias_pct > 100) or
       ($fake_pct   < 0) or ($fake_pct  > 100)) {
      error make { msg: $"Percentages must be within [0,100] (got: code=($code_pct) alias=($alias_pct) fake=($fake_pct))" }
   }

   let pct_sum = $code_pct + $alias_pct + $fake_pct
   if $pct_sum != 100 {
      error make { msg: $"Percentages must sum to 100 (got: ($pct_sum))" }
   }

   if ($code_path | path exists) != true {
      error make { msg: $"File not found: ($code_path)" }
   }
   if ($alias_path | path exists) != true {
      error make { msg: $"File not found: ($alias_path)" }
   }

   let code_limit  = ($code_pct  / 100 * $limit) | into int
   let alias_limit = ($alias_pct / 100 * $limit) | into int
   let fake_limit  = ($fake_pct  / 100 * $limit) | into int

   let codes =   open $code_path  | skip $code_offset  | get code  | take $code_limit
   let aliases = open $alias_path | skip $alias_offset | get alias | take $alias_limit

   let code_reqs    = $codes   | par-each {|code| $"GET /($code)" }
   let aliases_reqs = $aliases | par-each {|code| $"GET /($code)" }

   let fake_reqs = if $fake_pct > 0 { 0..$fake_limit | enumerate | par-each { $"GET /(gen_custom_id 32 $alpha)" } } else { [] }

   [$code_reqs, $aliases_reqs, $fake_reqs]
}

export def gen_shortener_reqs [
   --urls_path  (-i): path = ./scripts/data/urls.txt, # Path to list of input URLs
   --offset:        int = 0          # Line offset to start reading URLs
   --limit:         int = 100_000    # Total number of shortener requests to generate
   --code_pct:      int = 65         # Percentage of normal code generation
   --alias_pct:     int = 20         # Percentage of alias-based shortens
   --dup_code_pct:  int = 10         # Percentage of duplicate normal code requests
   --dup_alias_pct: int = 5          # Percentage of duplicate alias requests
]: nothing -> list<list<string>> {

   # --- Validate percentages: each in [0,100] and sum == 100 ----------------
   if (($code_pct      < 0) or ($code_pct      > 100) or
       ($alias_pct     < 0) or ($alias_pct     > 100) or
       ($dup_code_pct  < 0) or ($dup_code_pct  > 100) or
       ($dup_alias_pct < 0) or ($dup_alias_pct > 100)) {
      error make { msg: $"Percentages must be within [0,100] (got: code=($code_pct) alias=($alias_pct) dup_code=($dup_code_pct) dup_alias=($dup_alias_pct))" }
   }

   let pct_sum = $code_pct + $alias_pct + $dup_code_pct + $dup_alias_pct
   if $pct_sum != 100 {
      error make { msg: $"Percentages must sum to 100 (got: ($pct_sum))" }
   }


   if ( $urls_path | path exists) != true {
      error make { msg: $"File not found: ($urls_path)" }
   }

   # --- Compute counts -----------------------------------------------------
   let code_limit      = ($code_pct      / 100 * $limit) | into int
   let alias_limit     = ($alias_pct     / 100 * $limit) | into int
   let dup_code_limit  = ($dup_code_pct  / 100 * $limit) | into int
   let dup_alias_limit = ($dup_alias_pct / 100 * $limit) | into int

   let urls = open $urls_path | lines | skip $offset | take ($code_limit + $alias_limit)

   let codes_reqs = if $code_pct > 0 {
      ($urls | take $code_limit | par-each { |url| $"POST /api/shorten ($url)" })
   } else {
      []
   }

   let aliases_reqs = if $alias_pct > 0 {
      ($urls | skip $code_limit | take $alias_limit |
         par-each { |url| $"POST /api/shorten?alias=(gen_custom_id 32 $alpha) ($url)" })
   } else {
      []
   }

   let dup_code_reqs    = (if $dup_code_pct  > 0 { $codes_reqs   | shuffle | take $dup_code_limit  } else { [] })
   let dup_aliases_reqs = (if $dup_alias_pct > 0 { $aliases_reqs | shuffle | take $dup_alias_limit } else { [] })

   [$codes_reqs, $aliases_reqs, $dup_code_reqs, $dup_aliases_reqs]
}

export def main [
   --limit:         int = 100_000    # Total number of requests to generate
   --output_path:   path = ./tests/perf/data/reqs_list.txt # Output file path
   --shortener_pct: int = 10         # Percentage of shortener requests
   --redirect_only                  # Generate redirect requests only
   --shortener_only                 # Generate shortener requests only

   --code_path:    path = ./scripts/data/in_db/primary_codes_no_alias.csv # Path to primary codes CSV
   --alias_path:   path = ./scripts/data/in_db/aliases.csv                # Path to alias CSV
   --urls_path:    path = ./scripts/data/urls.txt,                        # Path to URL list

   --re_code_offset:  int = 0        # Redirect codes offset
   --re_alias_offset: int = 0        # Redirect aliases offset
   --re_code_pct:     int = 75       # Redirect primary codes percentage
   --re_alias_pct:    int = 15       # Redirect aliases percentage
   --re_fake_pct:     int = 10       # Redirect fake request percentage

   --sh_offset:         int = 0      # Shortener start offset
   --sh_code_pct:       int = 65     # Shortener primary codes percentage
   --sh_alias_pct:      int = 20     # Shortener aliases percentage
   --sh_dup_code_pct:   int = 10     # Shortener duplicate code requests percentage
   --sh_dup_alias_pct:  int = 5      # Shortener duplicate alias requests percentage
]: nothing -> nothing {
   
   let reqs: list<string> =  if $redirect_only and (not $shortener_only) {
      (gen_redirect_reqs
         --limit  $limit
         --code_path $code_path
         --alias_path $alias_path
         --code_offset  $re_code_offset
         --alias_offset $re_alias_offset
         --code_pct  $re_code_pct 
         --alias_pct $re_alias_pct
         --fake_pct  $re_fake_pct
      )
   } else if $shortener_only and (not $redirect_only) {
      (gen_shortener_reqs
         --limit  $limit
         --urls_path $urls_path
         --offset $sh_offset
         --code_pct  $sh_code_pct 
         --alias_pct $sh_alias_pct
         --dup_code_pct $sh_dup_code_pct
         --dup_alias_pct $sh_dup_alias_pct
      )
   } else {
      let redirect_pct = (100 - $shortener_pct)
      let redirect_limit = ($redirect_pct  / 100 * $limit | into int)
      let shortner_limit = ($shortener_pct / 100 * $limit | into int)

      let re_reps = (gen_redirect_reqs
         --limit  $redirect_limit
         --code_path $code_path
         --alias_path $alias_path
         --code_offset  $re_code_offset
         --alias_offset $re_alias_offset
         --code_pct  $re_code_pct 
         --alias_pct $re_alias_pct
         --fake_pct  $re_fake_pct
      )
      let sh_reqs = (gen_shortener_reqs
         --limit  $shortner_limit
         --urls_path $urls_path
         --offset $sh_offset
         --code_pct  $sh_code_pct 
         --alias_pct $sh_alias_pct
         --dup_code_pct $sh_dup_code_pct
         --dup_alias_pct $sh_dup_alias_pct
      )

      [($re_reps | flatten), ($sh_reqs | flatten)]
   }

   ($reqs | flatten | shuffle) | save -f $output_path
}
