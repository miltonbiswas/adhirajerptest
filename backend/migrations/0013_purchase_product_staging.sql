BEGIN;

-- Draft purchase lines may stage a new product without inserting it into
-- Product Master until the purchase is posted.
ALTER TABLE public.purchase_items
    ALTER COLUMN product_id DROP NOT NULL;

ALTER TABLE public.purchase_items
    ADD COLUMN product_payload JSONB;

ALTER TABLE public.purchase_items
    ADD CONSTRAINT chk_purchase_item_product_or_payload
    CHECK (
        product_id IS NOT NULL
        OR (
            product_payload IS NOT NULL
            AND product_payload ->> 'action' = 'CREATE'
        )
    );

COMMIT;
