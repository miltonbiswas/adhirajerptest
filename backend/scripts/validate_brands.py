from collections import Counter
from pathlib import Path
from openpyxl import load_workbook
import re

path = Path(r"F:\adhirajpos\Documents\brand lists.xlsx")
wb = load_workbook(path, read_only=True, data_only=True)
ws = wb.active

brands = []
blank_rows = []

for row_number, row in enumerate(ws.iter_rows(min_col=1, max_col=1, values_only=True), start=1):
    value = row[0]

    if value is None or not str(value).strip():
        blank_rows.append(row_number)
        continue

    original = str(value)
    cleaned = re.sub(r"\s+", " ", original).strip()
    brands.append((row_number, original, cleaned, cleaned.casefold()))

wb.close()

counts = Counter(item[3] for item in brands)
duplicate_groups = {name: count for name, count in counts.items() if count > 1}
short_entries = [item for item in brands if len(item[2]) <= 5]

print("Total Excel rows:", len(brands) + len(blank_rows))
print("Nonblank entries:", len(brands))
print("Blank rows:", len(blank_rows))
print("Unique normalized names:", len(counts))
print("Duplicate name groups:", len(duplicate_groups))
print("Extra duplicate rows:", sum(n - 1 for n in duplicate_groups.values()))
print("Entries of 5 characters or fewer:", len(short_entries))

print("\nFirst 25 short entries for manual review:")
for row_number, original, cleaned, _ in short_entries[:25]:
    print(f"Excel row {row_number}: {original!r}")

print("\nFirst 15 duplicate groups:")
for name, count in list(duplicate_groups.items())[:15]:
    print(f"{name!r}: {count} occurrences")
