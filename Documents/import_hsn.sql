\set ON_ERROR_STOP on
BEGIN;

CREATE TEMP TABLE hsn_import_stage (
    hsn_code VARCHAR(8) PRIMARY KEY,
    description TEXT NOT NULL DEFAULT '',
    gst_rate NUMERIC(5,2) NOT NULL,
    cgst_rate NUMERIC(5,2) NOT NULL,
    sgst_rate NUMERIC(5,2) NOT NULL,
    igst_rate NUMERIC(5,2) NOT NULL
) ON COMMIT DROP;

\copy hsn_import_stage (hsn_code, gst_rate, cgst_rate, sgst_rate, igst_rate) FROM 'F:/adhirajpos/Documents/hsn_normalized_import.csv' WITH (FORMAT csv, HEADER true);

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.hsn_master h
        JOIN hsn_import_stage s ON s.hsn_code = h.hsn_code
    ) THEN
        RAISE EXCEPTION 'Import cancelled: one or more HSN codes already exist in hsn_master.';
    END IF;
END
$$;

INSERT INTO public.hsn_master (
    hsn_code, description, gst_rate, cgst_rate, sgst_rate, igst_rate
)
SELECT
    hsn_code, description, gst_rate, cgst_rate, sgst_rate, igst_rate
FROM hsn_import_stage;

SELECT COUNT(*) AS imported_rows FROM hsn_import_stage;

COMMIT;
