use helpers.nu [timestamp exists_and_nonempty]


def try_get [file: path]: nothing -> path {
    if not (exists_and_nonempty $file) {
        return ""
    }
        
    glob $file | last
}

def ensure_urls_in_db_csv []: nothing -> path {
    let ts = timestamp
    mut csv = try_get ./scripts/data/in_db/urls*.csv

    if ($csv == "") {
        print "→ generating urls*.csv via get_ulr_data_from_db.nu"
        mkdir "./scripts/data" | ignore
        nu ./scripts/get_ulr_data_from_db.nu

        $csv = try_get ./scripts/data/in_db/urls*.csv
    }

    $csv
}

def generate_db_extra [csv: string]: nothing -> path {
    mkdir "./tests/perf/data" | ignore
    nu ./scripts/gen_short_id.nu -i $csv
    let out = (glob "tests/perf/data/db_code_extra_*.txt" | last)
    if $out == null { return "" } else { $out }
}


export def main []: nothing -> path {
    mut file = (try_get "tests/perf/data/db_code_extra_*.txt")

    if ($file == "") {
        let csv = ensure_urls_in_db_csv
        print "→ generating db_code_extra_* via gen_short_id.nu"
        $file = generate_db_extra $csv
    } else if not (exists_and_nonempty $file) {
        error make { msg: "Failed to prepare db_code_extra_* file." }
    }

    $file
}