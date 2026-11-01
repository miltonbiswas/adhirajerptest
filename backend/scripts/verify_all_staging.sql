SELECT
    'Products Staging' AS table_name,
    COUNT(*) AS total_records,
    COUNT(DISTINCT source_row) AS unique_source_rows
FROM public.product_import_staging

UNION ALL

SELECT
    'Brands Staging',
    COUNT(*),
    COUNT(DISTINCT source_row)
FROM public.brand_import_staging

UNION ALL

SELECT
    'Compositions Staging',
    COUNT(*),
    COUNT(DISTINCT source_row)
FROM public.composition_import_staging

UNION ALL

SELECT
    'Products Master',
    COUNT(*),
    NULL::BIGINT
FROM public.products

UNION ALL

SELECT
    'Brands Master',
    COUNT(*),
    NULL::BIGINT
FROM public.brands

UNION ALL

SELECT
    'Compositions Master',
    COUNT(*),
    NULL::BIGINT
FROM public.compositions;
