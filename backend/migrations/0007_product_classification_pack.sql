BEGIN;

ALTER TABLE products
    ADD COLUMN IF NOT EXISTS schedule_class VARCHAR(10)
        NOT NULL DEFAULT 'OTC',
    ADD COLUMN IF NOT EXISTS is_narcotic BOOLEAN
        NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS units_per_pack NUMERIC(14,3)
        NOT NULL DEFAULT 1;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_products_schedule_class'
          AND conrelid = 'products'::regclass
    ) THEN
        ALTER TABLE products
            ADD CONSTRAINT chk_products_schedule_class
            CHECK (schedule_class IN ('OTC', 'H', 'H1', 'X'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_products_units_per_pack'
          AND conrelid = 'products'::regclass
    ) THEN
        ALTER TABLE products
            ADD CONSTRAINT chk_products_units_per_pack
            CHECK (units_per_pack > 0);
    END IF;
END
$$;

CREATE INDEX IF NOT EXISTS idx_products_schedule_class
    ON products (schedule_class);

COMMIT;