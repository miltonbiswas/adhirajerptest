BEGIN;

-- Link each product to its brand master record.
ALTER TABLE products
ADD COLUMN brand_id UUID;

-- Prevent deleting a brand while products reference it.
ALTER TABLE products
ADD CONSTRAINT fk_products_brand
FOREIGN KEY (brand_id)
REFERENCES brands(id)
ON UPDATE RESTRICT
ON DELETE RESTRICT;

-- Speed up brand-based product searches.
CREATE INDEX idx_products_brand_id
ON products (brand_id);

COMMIT;