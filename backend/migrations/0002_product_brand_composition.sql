BEGIN;

-- ============================================
-- 1. BRANDS MASTER
-- ============================================

CREATE TABLE brands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    name TEXT NOT NULL,
    normalized_name TEXT GENERATED ALWAYS AS (
        lower(regexp_replace(trim(name), '\s+', ' ', 'g'))
    ) STORED,

    manufacturer_id UUID REFERENCES manufacturers(id)
        ON UPDATE RESTRICT
        ON DELETE RESTRICT,

    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT brands_name_not_empty
        CHECK (length(trim(name)) > 0)
);

CREATE UNIQUE INDEX uq_brands_normalized_name
    ON brands (normalized_name);

CREATE INDEX idx_brands_name
    ON brands (name);

CREATE INDEX idx_brands_manufacturer_id
    ON brands (manufacturer_id);


-- ============================================
-- 2. COMPOSITIONS MASTER
-- ============================================

CREATE TABLE compositions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    name TEXT NOT NULL,
    normalized_name TEXT GENERATED ALWAYS AS (
        lower(regexp_replace(trim(name), '\s+', ' ', 'g'))
    ) STORED,

    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT compositions_name_not_empty
        CHECK (length(trim(name)) > 0)
);

CREATE UNIQUE INDEX uq_compositions_normalized_name
    ON compositions (normalized_name);

CREATE INDEX idx_compositions_name
    ON compositions (name);


-- ============================================
-- 3. PRODUCT-COMPOSITION MAPPING
-- Supports multiple ingredients per product
-- ============================================

CREATE TABLE product_compositions (
    product_id UUID NOT NULL
        REFERENCES products(id)
        ON UPDATE RESTRICT
        ON DELETE RESTRICT,

    composition_id UUID NOT NULL
        REFERENCES compositions(id)
        ON UPDATE RESTRICT
        ON DELETE RESTRICT,

    strength TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (product_id, composition_id)
);

CREATE INDEX idx_product_compositions_composition_id
    ON product_compositions (composition_id);


COMMIT;
