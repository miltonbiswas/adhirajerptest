BEGIN;

CREATE TEMP TABLE composition_import_temp (
    source_row INTEGER,
    original_name TEXT,
    cleaned_name TEXT,
    normalized_name TEXT,
    review_status VARCHAR(30)
) ON COMMIT DROP;

\copy composition_import_temp (source_row, original_name, cleaned_name, normalized_name, review_status) FROM 'F:/adhirajpos/Documents/composition_staging.csv' WITH (FORMAT csv, HEADER true, ENCODING 'UTF8')

DO $$
BEGIN
    IF (SELECT COUNT(*) FROM composition_import_temp) <> 4063 THEN
        RAISE EXCEPTION 'Expected 4063 composition records; import cancelled.';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.composition_import_staging
        WHERE source_file = 'COmpositions.xlsx'
    ) THEN
        RAISE EXCEPTION 'Composition staging data already exists for this source. Import cancelled to prevent duplicates.';
    END IF;

    IF EXISTS (
        SELECT source_row
        FROM composition_import_temp
        GROUP BY source_row
        HAVING COUNT(*) > 1
    ) THEN
        RAISE EXCEPTION 'Duplicate source row numbers found; import cancelled.';
    END IF;
END $$;

INSERT INTO public.composition_import_staging (
    source_file,
    source_row,
    original_name,
    cleaned_name,
    normalized_name,
    review_status
)
SELECT
    'COmpositions.xlsx',
    source_row,
    original_name,
    cleaned_name,
    normalized_name,
    review_status
FROM composition_import_temp
ORDER BY source_row;

COMMIT;

SELECT
    COUNT(*) AS imported_records,
    COUNT(DISTINCT source_row) AS unique_source_rows,
    COUNT(DISTINCT normalized_name) AS unique_normalized_names,
    COUNT(*) FILTER (WHERE review_status = 'PENDING_REVIEW') AS pending_review
FROM public.composition_import_staging
WHERE source_file = 'COmpositions.xlsx';
