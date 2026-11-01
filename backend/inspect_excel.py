from pathlib import Path
from collections import defaultdict
from openpyxl import load_workbook, Workbook
from openpyxl.styles import Font, Alignment

DOCUMENTS = Path(r"F:\adhirajpos\Documents")

FILES = [
    {
        "source": "brand lists.xlsx",
        "output": "brand_duplicate_report.xlsx",
        "title": "Brand",
    },
    {
        "source": "COmpositions.xlsx",
        "output": "composition_duplicate_report.xlsx",
        "title": "Composition",
    },
]


def normalize(value):
    if value is None:
        return ""
    return " ".join(str(value).split())


def inspect_file(source_file, output_file, title):
    source_path = DOCUMENTS / source_file
    output_path = DOCUMENTS / output_file

    if not source_path.exists():
        raise FileNotFoundError(f"Source file not found: {source_path}")

    workbook = load_workbook(
        filename=source_path,
        read_only=True,
        data_only=True,
    )

    sheet = workbook.active
    groups = defaultdict(list)
    total_rows = 0
    blank_rows = 0

    print(f"\nReading {source_file}...")

    for row_number, row in enumerate(
        sheet.iter_rows(values_only=True),
        start=1,
    ):
        value = row[0] if row else None

        if value is None or not str(value).strip():
            blank_rows += 1
            continue

        original = str(value)
        cleaned = normalize(original)

        groups[cleaned.casefold()].append({
            "row": row_number,
            "original": original,
            "normalized": cleaned,
        })

        total_rows += 1

    workbook.close()

    duplicates = {
        name: entries
        for name, entries in groups.items()
        if len(entries) > 1
    }

    excess_rows = sum(
        len(entries) - 1
        for entries in duplicates.values()
    )

    report = Workbook()
    summary = report.active
    summary.title = "Summary"

    summary.append(["Metric", "Count"])
    summary.append(["List type", title])
    summary.append(["Total non-blank rows", total_rows])
    summary.append(["Blank rows", blank_rows])
    summary.append(["Unique normalized names", len(groups)])
    summary.append(["Duplicate groups", len(duplicates)])
    summary.append(["Excess duplicate rows", excess_rows])

    details = report.create_sheet("Duplicate Details")
    details.append([
        "Normalized Name",
        "Original Excel Row",
        "Original Name",
        "Occurrences in Group",
        "Review Status",
    ])

    for normalized_name, entries in sorted(duplicates.items()):
        for entry in entries:
            details.append([
                normalized_name,
                entry["row"],
                entry["original"],
                len(entries),
                "REVIEW REQUIRED",
            ])

    for ws in report.worksheets:
        ws.freeze_panes = "A2"
        ws.auto_filter.ref = ws.dimensions

        for cell in ws[1]:
            cell.font = Font(bold=True)
            cell.alignment = Alignment(wrap_text=True)

        for column_cells in ws.columns:
            max_length = max(
                (
                    len(str(cell.value or ""))
                    for cell in list(column_cells)[:500]
                ),
                default=10,
            )
            ws.column_dimensions[
                column_cells[0].column_letter
            ].width = min(max(max_length + 2, 15), 55)

    report.save(output_path)

    print(f"Total rows: {total_rows}")
    print(f"Unique normalized names: {len(groups)}")
    print(f"Duplicate groups: {len(duplicates)}")
    print(f"Excess duplicate rows: {excess_rows}")
    print(f"Report saved: {output_path}")


def main():
    for file_info in FILES:
        inspect_file(
            file_info["source"],
            file_info["output"],
            file_info["title"],
        )

    print("\nBoth reports generated successfully.")
    print("Original Excel files were not modified.")
    print("No database changes were made.")
    print("products.txt was not used.")


if __name__ == "__main__":
    main()