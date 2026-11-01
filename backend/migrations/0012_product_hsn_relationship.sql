BEGIN;

ALTER TABLE public.products
    ADD CONSTRAINT fk_products_hsn
    FOREIGN KEY (hsn_code)
    REFERENCES public.hsn_master (hsn_code)
    ON UPDATE CASCADE
    ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_products_hsn_code
    ON public.products (hsn_code)
    WHERE hsn_code IS NOT NULL;

COMMIT;