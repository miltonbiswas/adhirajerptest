BEGIN;

ALTER TABLE purchases
    ADD COLUMN tax_mode VARCHAR(10) NOT NULL DEFAULT 'EXCLUSIVE',
    ADD COLUMN tax_type VARCHAR(12) NOT NULL DEFAULT 'INTRA_STATE';

ALTER TABLE purchases
    ADD CONSTRAINT chk_purchases_tax_mode
        CHECK (tax_mode IN ('INCLUSIVE', 'EXCLUSIVE')),
    ADD CONSTRAINT chk_purchases_tax_type
        CHECK (tax_type IN ('INTRA_STATE', 'INTER_STATE'));

COMMIT;