def main [
    --output_path (-o): path = "urls.txt" # output urls.txt path location
] {
    curl -L https://tranco-list.eu/download/daily/tranco_4N3XX-1m.csv.zip -o top-1m.csv.zip

    let filename = "top-1m.csv"

    # # 2) Unzip to a single CSV file
    unzip -p "./top-1m.csv.zip" | save -f $filename

    open --raw $filename | $"id,url(char nl)($in)" |
    from csv | get url | par-each { $"https://($in)" } | save --force $output_path
};