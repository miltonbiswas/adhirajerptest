import csv
from pathlib import Path
from openpyxl import load_workbook

SOURCE_FILE = Path(r"F:\adhirajpos\Documents\prodd.xlsx")
OUTPUT_FILE = Path(r"F:\adhirajpos\Documents\product_staging.csv")


def normalize(value):
    return " ".join(value.split()).casefold()


def main():
    if not SOURCE_FILE.exists():
        raise FileNotFoundError(f"Source file not found: {SOURCE_FILE}")

    if OUTPUT_FILE.exists():
        raise FileExistsError(
            f"Staging file already exists: {OUTPUT_FILE}\n"
            "Review it before replacing it."
        )

    workbook = load_workbook(
        SOURCE_FILE,
        read_only=True,
        data_only=True,
    )

    total = 0

    try:
        sheet = workbook.active

        with OUTPUT_FILE.open(
            "w",
            newline="",
            encoding="utf-8-sig",
        ) as file:
            writer = csv.DictWriter(
                file,
                fieldnames=[
                    "source_row",
                    "original_name",
                    "cleaned_name",
                    "sku",
                    "normalized_name",
                    "review_status",
                ],
            )
            writer.writeheader()

            for row_number, row in enumerate(
                sheet.iter_rows(values_only=True),
                start=1,
            ):
                value = row[0] if row else None

                if value is None or not str(value).strip():
                    raise ValueError(
                        f"Unexpected blank entry at Excel row {row_number}"
                    )

                if not isinstance(value, str):
                    raise ValueError(
                        f"Non-text product name at Excel row {row_number}"
                    )

                cleaned_name = " ".join(value.split())

                if not cleaned_name:
                    raise ValueError(
                        f"Empty cleaned name at Excel row {row_number}"
                    )

                total += 1

                writer.writerow({
                    "source_row": row_number,
                    "original_name": value,
                    "cleaned_name": cleaned_name,
                    "sku": f"ADP-{total:06d}",
                    "normalized_name": normalize(cleaned_name),
                    "review_status": "PENDING_REVIEW",
                })

    finally:
        workbook.close()

    print("Product staging preparation complete.")
    print(f"Source records: {total}")
    print(f"Staging file: {OUTPUT_FILE}")
    print("Original Excel file: unchanged")
    print("Database changes: none")
    print("Duplicate products: preserved separately")


if __name__ == "__main__":
    main()