import csv
import re
import sys
from pathlib import Path
from openpyxl import load_workbook

source = Path(r"F:\adhirajpos\Documents\COmpositions.xlsx")
output = Path(r"F:\adhirajpos\Documents\composition_staging.csv")

if not source.is_file():
    sys.exit(f"ERROR: Source file not found: {source}")

wb = load_workbook(source, read_only=True, data_only=True)
ws = wb.active

count = 0

with output.open("w", encoding="utf-8-sig", newline="") as f:
    writer = csv.writer(f)
    writer.writerow([
        "source_row",
        "original_name",
        "cleaned_name",
        "normalized_name",
        "review_status",
    ])

    for row_number, row in enumerate(
        ws.iter_rows(min_col=1, max_col=1, values_only=True), start=1
    ):
        value = row[0]

        if value is None or not str(value).strip():
            continue

        original = str(value)
        cleaned = re.sub(r"\s+", " ", original).strip()
        normalized = cleaned.casefold()

        writer.writerow([
            row_number,
            original,
            cleaned,
            normalized,
            "PENDING_REVIEW",
        ])
        count += 1

wb.close()

print("Source file:", source.name)
print("CSV created:", output)
print("Records prepared:", count)
print("Original Excel file was not modified.")

if count != 4063:
    sys.exit("ERROR: Expected 4063 records; do not import this CSV.")
