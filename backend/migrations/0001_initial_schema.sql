BEGIN;

CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(120) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_categories_name UNIQUE (name)
);

CREATE TABLE manufacturers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    contact_person VARCHAR(150),
    phone VARCHAR(20),
    email VARCHAR(254),
    address TEXT,
    gstin VARCHAR(15),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_manufacturers_name UNIQUE (name)
);

CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku VARCHAR(80) NOT NULL,
    name VARCHAR(250) NOT NULL,
    generic_name VARCHAR(250),
    category_id UUID REFERENCES categories(id) ON DELETE RESTRICT,
    manufacturer_id UUID REFERENCES manufacturers(id) ON DELETE RESTRICT,
    brand_name VARCHAR(150),
    dosage_form VARCHAR(80),
    strength VARCHAR(80),
    pack_size VARCHAR(80),
    unit VARCHAR(30) NOT NULL DEFAULT 'PCS',
    hsn_code VARCHAR(8),
    gst_rate NUMERIC(5,2) NOT NULL DEFAULT 0,
    barcode VARCHAR(100),
    is_prescription_required BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_products_sku UNIQUE (sku),
    CONSTRAINT chk_products_gst_rate
        CHECK (gst_rate >= 0 AND gst_rate <= 100)
);

CREATE INDEX idx_products_name ON products (name);
CREATE INDEX idx_products_generic_name ON products (generic_name);
CREATE INDEX idx_products_category_id ON products (category_id);
CREATE INDEX idx_products_manufacturer_id ON products (manufacturer_id);
CREATE INDEX idx_products_barcode ON products (barcode)
    WHERE barcode IS NOT NULL;

CREATE TABLE product_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    batch_number VARCHAR(100) NOT NULL,
    manufacturing_date DATE,
    expiry_date DATE,
    purchase_rate NUMERIC(14,4) NOT NULL DEFAULT 0,
    mrp NUMERIC(14,4) NOT NULL DEFAULT 0,
    selling_rate NUMERIC(14,4) NOT NULL DEFAULT 0,
    quantity_on_hand NUMERIC(14,3) NOT NULL DEFAULT 0,
    reorder_level NUMERIC(14,3) NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_product_batch UNIQUE (product_id, batch_number),
    CONSTRAINT chk_batch_purchase_rate CHECK (purchase_rate >= 0),
    CONSTRAINT chk_batch_mrp CHECK (mrp >= 0),
    CONSTRAINT chk_batch_selling_rate CHECK (selling_rate >= 0),
    CONSTRAINT chk_batch_quantity CHECK (quantity_on_hand >= 0),
    CONSTRAINT chk_batch_reorder_level CHECK (reorder_level >= 0),
    CONSTRAINT chk_batch_dates CHECK (
        manufacturing_date IS NULL
        OR expiry_date IS NULL
        OR manufacturing_date <= expiry_date
    )
);

CREATE INDEX idx_batches_product_id ON product_batches (product_id);

CREATE INDEX idx_batches_fefo
    ON product_batches (expiry_date, product_id)
    WHERE is_active = TRUE;

CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(250) NOT NULL,
    contact_person VARCHAR(150),
    phone VARCHAR(20),
    email VARCHAR(254),
    gstin VARCHAR(15),
    drug_license_number VARCHAR(100),
    address TEXT,
    opening_balance NUMERIC(14,2) NOT NULL DEFAULT 0,
    opening_balance_type VARCHAR(10) NOT NULL DEFAULT 'PAYABLE',
    credit_days INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_supplier_opening_balance_type
        CHECK (opening_balance_type IN ('PAYABLE', 'ADVANCE')),
    CONSTRAINT chk_supplier_credit_days CHECK (credit_days >= 0)
);

CREATE INDEX idx_suppliers_name ON suppliers (name);
CREATE INDEX idx_suppliers_phone ON suppliers (phone);

CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(250) NOT NULL,
    phone VARCHAR(20),
    email VARCHAR(254),
    gstin VARCHAR(15),
    address TEXT,
    opening_balance NUMERIC(14,2) NOT NULL DEFAULT 0,
    opening_balance_type VARCHAR(10) NOT NULL DEFAULT 'RECEIVABLE',
    credit_limit NUMERIC(14,2) NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_customer_opening_balance_type
        CHECK (opening_balance_type IN ('RECEIVABLE', 'ADVANCE')),
    CONSTRAINT chk_customer_credit_limit CHECK (credit_limit >= 0)
);

CREATE INDEX idx_customers_name ON customers (name);
CREATE INDEX idx_customers_phone ON customers (phone);

COMMIT;