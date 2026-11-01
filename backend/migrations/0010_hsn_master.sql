CREATE TABLE IF NOT EXISTS hsn_master (
    hsn_code VARCHAR(8) PRIMARY KEY,
    description TEXT NOT NULL DEFAULT '',
    gst_rate NUMERIC(5,2) NOT NULL DEFAULT 0,
    cgst_rate NUMERIC(5,2) NOT NULL DEFAULT 0,
    sgst_rate NUMERIC(5,2) NOT NULL DEFAULT 0,
    igst_rate NUMERIC(5,2) NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT hsn_master_gst_rate_check
        CHECK (gst_rate >= 0 AND gst_rate <= 100),

    CONSTRAINT hsn_master_cgst_rate_check
        CHECK (cgst_rate >= 0 AND cgst_rate <= 100),

    CONSTRAINT hsn_master_sgst_rate_check
        CHECK (sgst_rate >= 0 AND sgst_rate <= 100),

    CONSTRAINT hsn_master_igst_rate_check
        CHECK (igst_rate >= 0 AND igst_rate <= 100)
);

CREATE INDEX IF NOT EXISTS idx_hsn_master_active
    ON hsn_master (hsn_code)
    WHERE is_active = TRUE;
