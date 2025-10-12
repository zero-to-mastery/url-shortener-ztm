use helpers.nu [timestamp exists_and_nonempty]

def ensure_urls_txt []: nothing -> string {
    let out = "./scripts/data/urls.txt"
    if not (exists_and_nonempty $out) {
        print "→ generating ./scripts/data/urls.txt via get_urls.nu"
        mkdir "./scripts/data" | ignore
        nu ./scripts/get_urls.nu -o $out
    }
    $out
}

def latest_url_repeat []: nothing -> path {
    let m = (glob "./tests/perf/data/url_repeat_*" | if ($in | is-empty) { "" } else { last })

    $m
}

def generate_url_repeat []: nothing -> string {
    mkdir "./tests/perf/data" | ignore
    nu ./scripts/gen_repeat_url.nu
}

export def main []: nothing -> path {
    mut file = latest_url_repeat

    if  not (exists_and_nonempty $file) {
        ensure_urls_txt
        print "→ generating url_repeat_* via gen_*_url.nu"
        generate_url_repeat
        $file = latest_url_repeat
    }
    if ($file == "" or not (exists_and_nonempty $file)) {
        error make { msg: "Failed to prepare url_repeat_* file." }
    }

    $file
}

