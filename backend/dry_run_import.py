from pathlib import Path
from collections import Counter

from openpyxl import load_workbook


DOCUMENTS = Path(r"F:\adhirajpos\Documents")

FILES = {
    "products": "prodd.xlsx",
    "brands": "brand lists.xlsx",
    "compositions": "COmpositions.xlsx",
}


def normalize(value):
    """Normalize whitespace and case for matching only."""
    return " ".join(str(value).split()).casefold()


def read_names(filename):
    path = DOCUMENTS / filename

    if not path.exists():
        raise FileNotFoundError(f"Excel file not found: {path}")

    workbook = load_workbook(
        path,
        read_only=True,
        data_only=True,
    )

    try:
        sheet = workbook.active
        names = []

        for row_number, row in enumerate(
            sheet.iter_rows(values_only=True),
            start=1,
        ):
            value = row[0] if row else None

            if value is None or not str(value).strip():
                continue

            if not isinstance(value, str):
                raise ValueError(
                    f"{filename}, row {row_number}: "
                    f"expected text, got {type(value).__name__}"
                )

            original_name = value
            cleaned_name = " ".join(value.split())

            if not cleaned_name:
                continue

            names.append({
                "source_row": row_number,
                "original_name": original_name,
                "cleaned_name": cleaned_name,
                "normalized_name": normalize(cleaned_name),
            })

        return names

    finally:
        workbook.close()


def inspect_dataset(label, filename):
    records = read_names(filename)

    counts = Counter(
        record["normalized_name"]
        for record in records
    )

    duplicate_groups = sum(
        1 for count in counts.values()
        if count > 1
    )

    excess_rows = sum(
        count - 1
        for count in counts.values()
        if count > 1
    )

    print(f"\n--- {label.upper()} ---")
    print(f"Source file: {filename}")
    print(f"Valid non-blank records: {len(records)}")
    print(f"Unique normalized names: {len(counts)}")
    print(f"Duplicate groups: {duplicate_groups}")
    print(f"Excess duplicate rows: {excess_rows}")

    if records:
        print("First 5 records:")
        for record in records[:5]:
            print(
                f"  Excel row {record['source_row']}: "
                f"{record['cleaned_name']}"
            )

    if label == "products":
        print("\nSKU plan:")
        print("  Internal SKU format: ADP-000001, ADP-000002, ...")
        print("  SKUs will be generated only during the actual import.")
        print("  Original product names will be preserved.")

    return records


def main():
    print("ADHIRAJPOS IMPORT DRY RUN")
    print("=" * 40)
    print("Mode: READ-ONLY")
    print("Database connection: NOT USED")
    print("Database inserts: NONE")
    print("Original Excel files: NOT MODIFIED")

    all_records = {}

    for label, filename in FILES.items():
        all_records[label] = inspect_dataset(label, filename)

    print("\n" + "=" * 40)
    print("DRY RUN COMPLETE")
    print("No database records were inserted or changed.")
    print("\nImportant:")
    print("- Duplicate names have not been merged.")
    print("- Products have not been linked to brands.")
    print("- Products have not been linked to compositions.")
    print("- Product attributes absent from Excel remain unknown.")


if __name__ == "__main__":
    main()