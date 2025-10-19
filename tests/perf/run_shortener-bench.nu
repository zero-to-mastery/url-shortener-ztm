use ../../scripts/gen_req_url.nu
use ../../scripts/get_ulr_data_from_db.nu 
use ../../scripts/helpers.nu [exists_and_nonempty]
use ../../scripts/get_urls.nu

export def main [
    --reset
    --prefix:string    = "shortener-bench"
    --report_path:path = ./reports
] {
    let ts = (date now | format date "%Y-%m-%d_%H:%M:%S")
    let fname = $"($report_path)/($prefix)_($ts)"
    
    rm -rf tests/perf/data/*

    if not (exists_and_nonempty ./scripts/data/urls.txt) {
        get_urls  -o ./scripts/data/urls.txt
    }

    let cfg = open ./configuration/base.yml
    let cmd = $"TRUNCATE urls, bloom_snapshots RESTART IDENTITY CASCADE;"
    ^psql ($cfg.database.url) -c ($cmd) 
    
    
    gen_req_url --limit 150_000 --shortener_only --sh_code_pct 100 --sh_alias_pct 0 --sh_dup_alias_pct 0 --sh_dup_code_pct 0
    with-env {
        BASE_URL: "http://localhost:8000"
        API_KEY: "e4125dd1-3d3e-43a1-bc9c-dc0ba12ad4b5"
        NO_REPORT: null
        OP_FILE: ("./tests/perf/data/reqs_list.txt" | path expand) # get full path otherwise k6 error
    } {
        k6 run ./tests/perf/shortener-bench.js
    }

    get_ulr_data_from_db --tables primary_codes_no_alias,aliases
    gen_req_url --limit 150_000 --sh_offset 150_001

    with-env {
        BASE_URL: "http://localhost:8000"
        API_KEY: "e4125dd1-3d3e-43a1-bc9c-dc0ba12ad4b5"
        K6_WEB_DASHBOARD: "true"
        REPORT_PREFIX: $"./reports/($prefix)_($ts)"
        K6_WEB_DASHBOARD_PERIOD: "3s"
        K6_WEB_DASHBOARD_EXPORT: $"($fname)_dashboard.html"
        OP_FILE: ("./tests/perf/data/reqs_list.txt" | path expand) 
    } {
        k6 run ./tests/perf/shortener-bench.js
    }
    print "report saved in:"
    print $"\t\t($fname)_summary.txt"
    print $"\t\t($fname)_summary.json"
    print $"\t\t($fname)_status_breakdown.txt"
    print $"\t\t($fname)_status_breakdown.json"
    print $"\t\t($fname)_dashboard.html"

    open  $"($fname)_summary.txt"
}