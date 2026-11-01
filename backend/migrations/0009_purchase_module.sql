BEGIN;

-- =========================================================
-- PURCHASE INVOICES
-- =========================================================

CREATE TABLE purchases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    purchase_number VARCHAR(50) NOT NULL UNIQUE,

    supplier_id UUID NOT NULL
        REFERENCES parties(id) ON DELETE RESTRICT,

    supplier_invoice_number VARCHAR(100) NOT NULL,
    invoice_date DATE NOT NULL,
    due_date DATE,

    status VARCHAR(12) NOT NULL DEFAULT 'DRAFT'
        CHECK (status IN ('DRAFT', 'POSTED', 'CANCELLED')),

    subtotal NUMERIC(16,2) NOT NULL DEFAULT 0,
    discount_amount NUMERIC(16,2) NOT NULL DEFAULT 0,
    taxable_amount NUMERIC(16,2) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(16,2) NOT NULL DEFAULT 0,
    other_charges NUMERIC(16,2) NOT NULL DEFAULT 0,
    round_off NUMERIC(16,2) NOT NULL DEFAULT 0,
    grand_total NUMERIC(16,2) NOT NULL DEFAULT 0,

    paid_amount NUMERIC(16,2) NOT NULL DEFAULT 0,

    notes TEXT,

    posted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_purchase_amounts CHECK (
        subtotal >= 0
        AND discount_amount >= 0
        AND taxable_amount >= 0
        AND tax_amount >= 0
        AND other_charges >= 0
        AND grand_total >= 0
        AND paid_amount >= 0
        AND paid_amount <= grand_total
    )
);

-- Duplicate supplier invoice prevention.
CREATE UNIQUE INDEX uq_purchase_supplier_invoice
    ON purchases (
        supplier_id,
        LOWER(BTRIM(supplier_invoice_number))
    )
    WHERE status <> 'CANCELLED';

CREATE INDEX idx_purchases_supplier_date
    ON purchases (supplier_id, invoice_date DESC);

CREATE INDEX idx_purchases_status_date
    ON purchases (status, invoice_date DESC);


-- =========================================================
-- PURCHASE LINE ITEMS
-- =========================================================

CREATE TABLE purchase_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    purchase_id UUID NOT NULL
        REFERENCES purchases(id) ON DELETE RESTRICT,

    line_number INTEGER NOT NULL,

    product_id UUID NOT NULL
        REFERENCES products(id) ON DELETE RESTRICT,

    batch_id UUID
        REFERENCES product_batches(id) ON DELETE RESTRICT,

    batch_number VARCHAR(100) NOT NULL,
    manufacturing_date DATE,
    expiry_date DATE,

    quantity NUMERIC(14,3) NOT NULL,
    free_quantity NUMERIC(14,3) NOT NULL DEFAULT 0,

    purchase_rate NUMERIC(14,4) NOT NULL,
    mrp NUMERIC(14,4) NOT NULL,
    selling_rate NUMERIC(14,4) NOT NULL DEFAULT 0,

    discount_percent NUMERIC(5,2) NOT NULL DEFAULT 0,
    discount_amount NUMERIC(16,2) NOT NULL DEFAULT 0,

    gst_rate NUMERIC(5,2) NOT NULL DEFAULT 0,
    taxable_amount NUMERIC(16,2) NOT NULL DEFAULT 0,
    cgst_amount NUMERIC(16,2) NOT NULL DEFAULT 0,
    sgst_amount NUMERIC(16,2) NOT NULL DEFAULT 0,
    igst_amount NUMERIC(16,2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(16,2) NOT NULL DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_purchase_item_line
        UNIQUE (purchase_id, line_number),

    CONSTRAINT chk_purchase_item_quantity
        CHECK (quantity > 0 AND free_quantity >= 0),

    CONSTRAINT chk_purchase_item_rates
        CHECK (
            purchase_rate >= 0
            AND mrp >= 0
            AND selling_rate >= 0
        ),

    CONSTRAINT chk_purchase_item_discount
        CHECK (
            discount_percent BETWEEN 0 AND 100
            AND discount_amount >= 0
        ),

    CONSTRAINT chk_purchase_item_gst
        CHECK (gst_rate BETWEEN 0 AND 100),

    CONSTRAINT chk_purchase_item_amounts
        CHECK (
            taxable_amount >= 0
            AND cgst_amount >= 0
            AND sgst_amount >= 0
            AND igst_amount >= 0
            AND total_amount >= 0
        ),

    CONSTRAINT chk_purchase_item_dates
        CHECK (
            manufacturing_date IS NULL
            OR expiry_date IS NULL
            OR manufacturing_date <= expiry_date
        )
);

CREATE INDEX idx_purchase_items_purchase
    ON purchase_items (purchase_id, line_number);

CREATE INDEX idx_purchase_items_product
    ON purchase_items (product_id);


-- =========================================================
-- SUPPLIER LEDGER
-- =========================================================

CREATE TABLE supplier_ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    supplier_id UUID NOT NULL
        REFERENCES parties(id) ON DELETE RESTRICT,

    purchase_id UUID
        REFERENCES purchases(id) ON DELETE RESTRICT,

    entry_type VARCHAR(20) NOT NULL
        CHECK (
            entry_type IN (
                'PURCHASE',
                'PAYMENT',
                'CREDIT_NOTE',
                'DEBIT_NOTE',
                'ADJUSTMENT'
            )
        ),

    debit_amount NUMERIC(16,2) NOT NULL DEFAULT 0,
    credit_amount NUMERIC(16,2) NOT NULL DEFAULT 0,

    reference_number VARCHAR(100),
    notes TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_supplier_ledger_amounts CHECK (
        debit_amount >= 0
        AND credit_amount >= 0
        AND NOT (
            debit_amount > 0
            AND credit_amount > 0
        )
        AND (
            debit_amount > 0
            OR credit_amount > 0
        )
    )
);

CREATE INDEX idx_supplier_ledger_supplier_date
    ON supplier_ledger_entries (supplier_id, created_at DESC);

CREATE UNIQUE INDEX uq_supplier_ledger_purchase
    ON supplier_ledger_entries (purchase_id)
    WHERE entry_type = 'PURCHASE';


COMMIT;