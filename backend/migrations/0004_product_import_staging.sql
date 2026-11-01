CREATE TABLE product_import_staging (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    source_file TEXT NOT NULL DEFAULT 'prodd.xlsx',
    source_row INTEGER NOT NULL,

    original_name TEXT NOT NULL,
    cleaned_name TEXT NOT NULL,
    normalized_name TEXT NOT NULL,

    proposed_sku VARCHAR(50) NOT NULL,
    review_status VARCHAR(30) NOT NULL DEFAULT 'PENDING_REVIEW',

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_product_import_staging_source_row
        UNIQUE (source_file, source_row),

    CONSTRAINT uq_product_import_staging_proposed_sku
        UNIQUE (proposed_sku),

    CONSTRAINT chk_product_import_staging_names
        CHECK (
            LENGTH(BTRIM(original_name)) > 0
            AND LENGTH(BTRIM(cleaned_name)) > 0
            AND LENGTH(BTRIM(normalized_name)) > 0
        ),

    CONSTRAINT chk_product_import_staging_review_status
        CHECK (
            review_status IN (
                'PENDING_REVIEW',
                'APPROVED',
                'REJECTED'
            )
        )
);

CREATE INDEX idx_product_import_staging_normalized_name
    ON product_import_staging (normalized_name);

CREATE INDEX idx_product_import_staging_review_status
    ON product_import_staging (review_status);