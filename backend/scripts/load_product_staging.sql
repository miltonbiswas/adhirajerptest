\set ON_ERROR_STOP on

BEGIN;

CREATE TEMP TABLE product_import_temp (
    source_row INTEGER NOT NULL,
    original_name TEXT NOT NULL,
    cleaned_name TEXT NOT NULL,
    proposed_sku VARCHAR(50) NOT NULL,
    normalized_name TEXT NOT NULL,
    review_status VARCHAR(30) NOT NULL
) ON COMMIT DROP;

\copy product_import_temp (source_row, original_name, cleaned_name, proposed_sku, normalized_name, review_status) FROM 'F:/adhirajpos/Documents/product_staging.csv' WITH (FORMAT csv, HEADER true, ENCODING 'UTF8')

DO $$
BEGIN
    IF (SELECT COUNT(*) FROM product_import_temp) <> 77858 THEN
        RAISE EXCEPTION 'Expected 77858 CSV rows; import cancelled';
    END IF;

    IF EXISTS (SELECT 1 FROM public.product_import_staging) THEN
        RAISE EXCEPTION 'Staging table is not empty; import cancelled to prevent duplicates';
    END IF;
END $$;

INSERT INTO public.product_import_staging (
    source_file,
    source_row,
    original_name,
    cleaned_name,
    normalized_name,
    proposed_sku,
    review_status
)
SELECT
    'prodd.xlsx',
    source_row,
    original_name,
    cleaned_name,
    normalized_name,
    proposed_sku,
    review_status
FROM product_import_temp
ORDER BY source_row;

COMMIT;

SELECT
    COUNT(*) AS imported_records,
    COUNT(DISTINCT proposed_sku) AS unique_skus,
    COUNT(DISTINCT source_row) AS unique_source_rows,
    COUNT(*) FILTER (WHERE review_status = 'PENDING_REVIEW') AS pending_review
FROM public.product_import_staging;
