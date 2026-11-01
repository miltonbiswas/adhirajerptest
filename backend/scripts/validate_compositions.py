from collections import Counter
from pathlib import Path
from openpyxl import load_workbook
import re
import sys

path = Path(r"F:\adhirajpos\Documents\COmpositions.xlsx")

if not path.is_file():
    sys.exit(f"ERROR: Excel file not found: {path}")

wb = load_workbook(path, read_only=True, data_only=True)
ws = wb.active

entries = []
blank_rows = []

for row_number, row in enumerate(
    ws.iter_rows(min_col=1, max_col=1, values_only=True), start=1
):
    value = row[0]

    if value is None or not str(value).strip():
        blank_rows.append(row_number)
        continue

    original = str(value)
    cleaned = re.sub(r"\s+", " ", original).strip()
    normalized = cleaned.casefold()

    entries.append((row_number, original, cleaned, normalized))

wb.close()

counts = Counter(item[3] for item in entries)
duplicate_groups = {
    name: count for name, count in counts.items() if count > 1
}

print("Excel file:", path.name)
print("Total Excel rows:", len(entries) + len(blank_rows))
print("Nonblank entries:", len(entries))
print("Blank rows:", len(blank_rows))
print("Unique normalized names:", len(counts))
print("Duplicate name groups:", len(duplicate_groups))
print("Extra duplicate rows:", sum(n - 1 for n in duplicate_groups.values()))

print("\nFirst 10 composition entries:")
for row_number, original, cleaned, normalized in entries[:10]:
    print(f"Row {row_number}: {original!r}")

print("\nFirst 15 duplicate groups:")
for name, count in list(duplicate_groups.items())[:15]:
    print(f"{name!r}: {count} occurrences")

print("\nOriginal Excel file was not modified.")
