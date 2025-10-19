# 🌿 Use Nushell as the shell
set shell := ["nu", "-c"]

shebang := if os() == 'windows' {
	'nu.exe'
} else {
	'/usr/bin/env nu'
}

# ── 🧰 Base config (can be overridden in .env) ─────────────────────
BASE := "http://127.0.0.1:8000"
API_KEY := "e4125dd1-3d3e-43a1-bc9c-dc0ba12ad4b5"

# Default values for dev / release modes
DEV_RATE_LIMIT_DEFAULT := "false"
DEV_RUST_LOG_DEFAULT   := "debug"
REL_RATE_LIMIT_DEFAULT := "true"
REL_RUST_LOG_DEFAULT   := "warn"

# Values that can also be overridden via .env
APP_RATE_LIMITING__ENABLED := "true"
RUST_LOG := "warn"

# ── 🚀 Start service (release / dev) ───────────────────────────────
# Release: rate limiting ON, log = warn by default
start rate="true" log="warn":
    @echo "🟢 Starting service in release mode..."
    RUST_LOG={{log}} APP_RATE_LIMITING__ENABLED={{rate}} cargo run --release

# Dev: rate limiting OFF, log = debug by default
start-dev rate="false" log="debug":
    @echo "🟡 Starting service in dev mode..."
    RUST_LOG={{log}} APP_RATE_LIMITING__ENABLED={{rate}} cargo run

prepare-shorten-data:
	nu ./scripts/prepare_shorten_data.nu

prepare-redirect-data:
	nu ./scripts/prepare_redirect_data.nu

# ── 📊 Performance Tests - Shorten ────────────────────────────────
perf-shorten:
	#!{{shebang}}
	use '{{justfile_directory()}}/scripts/prepare_shorten_data.nu'

	let ts = (date now | format date "%Y-%m-%d_%H:%M:%S")
	let file = prepare_shorten_data
	let name = "shorten"
	let prefix = $"./reports/($ts)_($name)"
	mkdir reports | ignore

	print $"󱐋 Running shorten test → reports in ($prefix)_*"
	with-env {
		BASE: "{{BASE}}"
		API_KEY: "{{API_KEY}}"
		URL_FILE: $file
		REPORT_PREFIX: $prefix
		K6_WEB_DASHBOARD: "true"
		K6_WEB_DASHBOARD_PERIOD: "3s"
		K6_WEB_DASHBOARD_EXPORT: $"($prefix)_dashboard.html"
	} { ^k6 run ./tests/perf/shorten.js }
	print "report saved in:"
	print $"\t\t($prefix)_summary.txt"
	print $"\t\t($prefix)_summary.json"
	print $"\t\t($prefix)_dashboard.html"
	open $"($prefix)_summary.txt"

# ── 🔁 Performance Tests - Redirect ───────────────────────────────
perf-redirect:
	#!{{shebang}}
	use '{{justfile_directory()}}/scripts/prepare_redirect_data.nu'

	let ts = (date now | format date "%Y%m%d_%H%M%S")
	let file = prepare_redirect_data

	let name = "redirect"
	let prefix = $"./reports/($ts)_($name)"
	mkdir reports | ignore
	print $"󱐋 Running redirect test → reports in ($prefix)"

	with-env {
		BASE: "{{BASE}}"
		API_KEY: "{{API_KEY}}"
		REPORT_PREFIX: $prefix
		URL_FILE: $file
		K6_WEB_DASHBOARD: "true"
		K6_WEB_DASHBOARD_PERIOD: "3s"
		K6_WEB_DASHBOARD_EXPORT: $"($prefix)_dashboard.html"
	} { ^k6 run ./tests/perf/redirect.js }
	open $"($prefix)_summary.txt"

perf-shorten-bench:
	nu "./tests/perf/run_shortener-bench.nu"