job-data
==========================
Keep track of your job applications in a handy json file.

Install with `cargo install job-data`.
Run with `job-data -h
Use the tui with `job-data --tui` and press `?`

Previously we stored jobs into a csv file. If you want to keep your old data:
run 'cat my.csv | python -c 'import csv, json, sys; print(json.dumps([dict(r) for r in csv.DictReader(sys.stdin)]))'
and modify the LastActionDate values to be an array instead of a simple string.
