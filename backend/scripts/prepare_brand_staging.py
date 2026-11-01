import csv
import re
import sys
from collections import Counter
from pathlib import Path
from openpyxl import load_workbook

EXCEL_PATH = Path(r"F:\adhirajpos\Documents\brand lists.xlsx")
CSV_PATH = Path(r"F:\adhirajpos\Documents\brand_staging.csv")

if not EXCEL_PATH.is_file():
    sys.exit(f"ERROR: Excel file not found: {EXCEL_PATH}")

records = []

wb = load_workbook(EXCEL_PATH, read_only=True, data_only=True)
ws = wb.active

for row_number, row in enumerate(
    ws.iter_rows(min_col=1, max_col=1, values_only=True), start=1
):
    value = row[0]

    if value is None or not str(value).strip():
        sys.exit(f"ERROR: Blank brand at Excel row {row_number}")

    original = str(value)
    cleaned = re.sub(r"\s+", " ", original).strip()
    normalized = cleaned.casefold()

    records.append({
        "source_row": row_number,
        "original_name": original,
        "cleaned_name": cleaned,
        "normalized_name": normalized,
        "review_status": "PENDING_REVIEW",
    })

wb.close()

if len(records) != 2297:
    sys.exit(f"ERROR: Expected 2297 rows, found {len(records)}")

with CSV_PATH.open("w", encoding="utf-8", newline="") as f:
    writer = csv.DictWriter(
        f,
        fieldnames=[
            "source_row",
            "original_name",
            "cleaned_name",
            "normalized_name",
            "review_status",
        ],
    )
    writer.writeheader()
    writer.writerows(records)

print(f"Excel entries preserved: {len(records)}")
print(f"Unique normalized names: {len(set(r['normalized_name'] for r in records))}")
print(f"Duplicate rows preserved: {len(records) - len(set(r['normalized_name'] for r in records))}")
print(f"CSV created: {CSV_PATH}")
print("No database records inserted yet.")
