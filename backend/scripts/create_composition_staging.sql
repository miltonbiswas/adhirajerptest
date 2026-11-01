CREATE TABLE IF NOT EXISTS public.composition_import_staging (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    source_file TEXT NOT NULL DEFAULT 'COmpositions.xlsx',
    source_row INTEGER NOT NULL,
    original_name TEXT NOT NULL,
    cleaned_name TEXT NOT NULL,
    normalized_name TEXT NOT NULL,
    review_status VARCHAR(30) NOT NULL DEFAULT 'PENDING_REVIEW',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_composition_import_staging_source_row
        UNIQUE (source_file, source_row),

    CONSTRAINT chk_composition_import_staging_names
        CHECK (
            LENGTH(BTRIM(original_name)) > 0
            AND LENGTH(BTRIM(cleaned_name)) > 0
            AND LENGTH(BTRIM(normalized_name)) > 0
        ),

    CONSTRAINT chk_composition_import_staging_review_status
        CHECK (review_status IN ('PENDING_REVIEW', 'APPROVED', 'REJECTED'))
);

CREATE INDEX IF NOT EXISTS idx_composition_import_staging_normalized_name
    ON public.composition_import_staging (normalized_name);
