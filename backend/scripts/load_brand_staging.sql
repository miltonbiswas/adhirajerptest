\set ON_ERROR_STOP on

BEGIN;

CREATE TEMP TABLE brand_import_temp (
    source_row INTEGER NOT NULL,
    original_name TEXT NOT NULL,
    cleaned_name TEXT NOT NULL,
    normalized_name TEXT NOT NULL,
    review_status VARCHAR(30) NOT NULL
) ON COMMIT DROP;

\copy brand_import_temp (source_row, original_name, cleaned_name, normalized_name, review_status) FROM 'F:/adhirajpos/Documents/brand_staging.csv' WITH (FORMAT csv, HEADER true, ENCODING 'UTF8')

DO $$
BEGIN
    IF (SELECT COUNT(*) FROM brand_import_temp) <> 2297 THEN
        RAISE EXCEPTION 'Expected 2297 rows; import cancelled';
    END IF;

    IF EXISTS (SELECT 1 FROM public.brand_import_staging) THEN
        RAISE EXCEPTION 'Brand staging table is not empty; import cancelled to prevent duplicates';
    END IF;
END $$;

INSERT INTO public.brand_import_staging (
    source_file,
    source_row,
    original_name,
    cleaned_name,
    normalized_name,
    review_status
)
SELECT
    'brand lists.xlsx',
    source_row,
    original_name,
    cleaned_name,
    normalized_name,
    review_status
FROM brand_import_temp
ORDER BY source_row;

COMMIT;

SELECT
    COUNT(*) AS imported_records,
    COUNT(DISTINCT source_row) AS unique_source_rows,
    COUNT(DISTINCT normalized_name) AS unique_normalized_names,
    COUNT(*) FILTER (WHERE review_status = 'PENDING_REVIEW') AS pending_review
FROM public.brand_import_staging;
