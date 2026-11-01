import csv
import os
import subprocess
import sys
from pathlib import Path

CSV_PATH = Path(r"F:\adhirajpos\Documents\product_staging.csv")
PSQL = Path(r"C:\Program Files\PostgreSQL\18\bin\psql.exe")

if not CSV_PATH.is_file():
    sys.exit(f"ERROR: CSV file not found: {CSV_PATH}")

if not PSQL.is_file():
    sys.exit(f"ERROR: psql.exe not found: {PSQL}")

expected_headers = [
    "source_row",
    "original_name",
    "cleaned_name",
    "sku",
    "normalized_name",
    "review_status",
]

with CSV_PATH.open("r", encoding="utf-8-sig", newline="") as f:
    reader = csv.DictReader(f)

    if reader.fieldnames != expected_headers:
        sys.exit(f"ERROR: Unexpected CSV headers: {reader.fieldnames}")

    count = 0
    seen_skus = set()
    seen_rows = set()

    for line_number, row in enumerate(reader, start=2):
        if not all(row.get(key) is not None for key in expected_headers):
            sys.exit(f"ERROR: Missing field on CSV line {line_number}")

        if not row["original_name"].strip() or not row["cleaned_name"].strip():
            sys.exit(f"ERROR: Blank product name on CSV line {line_number}")

        if row["review_status"] != "PENDING_REVIEW":
            sys.exit(f"ERROR: Unexpected review status on CSV line {line_number}")

        if row["sku"] in seen_skus:
            sys.exit(f"ERROR: Duplicate SKU in CSV: {row['sku']}")

        if row["source_row"] in seen_rows:
            sys.exit(f"ERROR: Duplicate source row in CSV: {row['source_row']}")

        seen_skus.add(row["sku"])
        seen_rows.add(row["source_row"])
        count += 1

if count != 77858:
    sys.exit(f"ERROR: Expected 77858 CSV records; found {count}. No import performed.")

print(f"CSV validation passed: {count:,} records.")
print("Import has NOT started yet.")
print("Next step will load records into product_import_staging only.")
