# Dependencies: psql / sqlite3; in container mode, ensure the client exists inside the container.

export def main [
  --db_type: string     # sqlite | pg
  --db_url: string      # sqlite: file path or sqlite://... ; pg: postgres://...
  --db_yml: path        = "./configuration/base.yml"   # Optional YAML; CLI flags take precedence
  --output (-o): path   = "./scripts/data/in_db"             # Export directory
  --tables: string      = "urls"                       # Tables to export, comma-separated (supports schema.table)
  --container: string   = ""                           # Docker container ID/Name; empty = run on host
] {
  # Read YAML if present; CLI has priority
  let cfg = if ($db_yml | path exists) { open $db_yml } else { {} }

  let eff_type = (
    [ $db_type ($cfg.database.type? | default null) ]
    | compact | first | default "" | str downcase
  )
  let eff_url = (
    [ $db_url ($cfg.database.url? | default null) ]
    | compact | first | default ""
  )

  if $eff_type == "" { error make { msg: "Provide --db_type or set database.type in YAML" } }
  if $eff_url  == "" { error make { msg: "Provide --db_url or set database.url in YAML" } }

  mkdir $output | ignore

  # Normalize --tables (commas/whitespace supported)
  let tbls = if ($tables | str length) == 0 {
    []
  } else {
    $tables
    | str replace -a "," " "
    | split row " "
    | each { |t| $t | str trim }
    | where { |t| $t != "" }
  }

  match $eff_type {
    "sqlite" => { export-sqlite $eff_url $output $container $tbls }
    "pg" | "postgres" | "postgresql" => { export-pg $eff_url $output $container $tbls }
    _ => { error make { msg: $"Unsupported db_type: ($eff_type)" } }
  }
}

# ---------- Helpers ----------
def quote_ident [name: string] {
  # Escape inner double quotes for identifiers
  let inner = ($name | str replace --all '"' '""')
  $"\"($inner)\""
}

def sanitize_filename [s?: string] {
  let v = if $s == null { $in } else { $s }
  $v
  | into string
  | str replace --all "." "__"
  | str replace --all "/" "_"
  | str replace --all "\\" "_"
}

def escape_single_quote [s: string] {
  # For single-quoted literals in \copy TO 'path'
  $s | str replace --all "'" "''"
}

# ---------- SQLite ----------
def export-sqlite [
  url_or_path: string
  outdir: path
  container: string
  tables: list<any>
] {
  let db_path = if ($url_or_path | str starts-with "sqlite://") {
    $url_or_path | str replace -a "sqlite://" ""
  } else { $url_or_path }

  let use_dk = ($container | default "" | str length) > 0

  # Decide tables to export
  let target_tables = if ($tables | length) > 0 {
    $tables
  } else {
    let list_sql = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name;"
    if $use_dk {
      ^docker exec $container sqlite3 $db_path $list_sql | lines
    } else {
      ^sqlite3 $db_path $list_sql | lines
    }
  }

  if ($target_tables | length) == 0 {
    print "(SQLite) No exportable tables found"
    return
  }

  for t in $target_tables {
    let tname = ($t | into string)
    let fname = (sanitize_filename $tname) + ".csv"
    let outfile = ($outdir | path join $fname)
    let q = $"SELECT * FROM \"($tname)\";"

    if ($container | is-empty) {
      ^sqlite3 -header -csv $db_path $q | save --raw --force $outfile
    } else {
      ^docker exec $container sqlite3 -header -csv $db_path $q | save --raw --force $outfile
    }
    print $"[SQLite] Exported table: ($tname) -> ($outfile)"
  }
}

# ---------- Postgres ----------
def export-pg [
  pg_url: string
  outdir: path
  container: string
  tables: list<any>
] {
  let use_dk = ($container | default "" | str length) > 0
  print $outdir

  # Decide tables to export (schema optional)
  let target_tables = if ($tables | length) > 0 {
    $tables
  } else {
    # If not specified, export all non-system tables
    let list_sql = "SELECT schemaname||'.'||tablename FROM pg_catalog.pg_tables WHERE schemaname NOT IN ('pg_catalog','information_schema') ORDER BY schemaname, tablename;"
    if $use_dk {
      ^docker exec $container psql $pg_url -At -c $list_sql | lines
    } else {
      ^psql $pg_url -At -c $list_sql | lines
    }
  }

  if ($target_tables | length) == 0 {
    print "(Postgres) No exportable tables found"
    return
  }

  for raw in $target_tables {
    let st = ($raw | into string | str trim)

    # Parse schema.table or table
    let parts = ($st | split row "." | take 2)
    let schema = if ($parts | length) == 2 { ($parts | get 0) } else { "" }
    let table  = if ($parts | length) == 2 { ($parts | get 1) } else { ($parts | get 0) }

    let qschema = if ($schema | str length) > 0 { (quote_ident $schema) } else { "" }
    let qtable  = (quote_ident $table)
    let obj     = if ($qschema | str length) > 0 { $"($qschema).($qtable)" } else { $qtable }

    let fname   = ((if ($schema | str length) > 0 { $"($schema)_($table)" } else { $table }) | sanitize_filename) + ".csv"
    let outfile = ($outdir | path join $fname)
    let cmd = $"COPY \(SELECT * FROM ($obj)\) TO STDOUT WITH \(FORMAT CSV, HEADER\)"

    if not $use_dk {
      # Host: stream CSV to file; \copy writes client-side
      print $"Running: psql <url> -c ($cmd) | save --raw --force ($outfile)"
      ^psql ($pg_url) -c ($cmd) | save --raw --force ($outfile)
    } else {
      # Container: stream CSV via STDOUT back to host
      ^docker exec $container psql $pg_url -c $cmd | save --raw --force $outfile
    }

    print $"[Postgres] Exported table: ($table) -> ($outfile)"
  }
}
