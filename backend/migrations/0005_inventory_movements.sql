CREATE TABLE IF NOT EXISTS stock_movements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    product_id UUID NOT NULL
        REFERENCES products(id) ON DELETE RESTRICT,

    batch_id UUID NOT NULL
        REFERENCES product_batches(id) ON DELETE RESTRICT,

    movement_type VARCHAR(20) NOT NULL
        CHECK (movement_type IN ('IN', 'OUT', 'ADJUSTMENT')),

    quantity_change NUMERIC(14,3) NOT NULL,
    quantity_before NUMERIC(14,3) NOT NULL,
    quantity_after NUMERIC(14,3) NOT NULL,

    reason TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_stock_movement_quantities
        CHECK (
            quantity_change <> 0
            AND quantity_before >= 0
            AND quantity_after >= 0
        )
);

CREATE INDEX IF NOT EXISTS idx_stock_movements_product
    ON stock_movements (product_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_stock_movements_batch
    ON stock_movements (batch_id, created_at DESC);