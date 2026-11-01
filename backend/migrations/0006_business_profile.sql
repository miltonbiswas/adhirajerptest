BEGIN;

CREATE TABLE IF NOT EXISTS business_profile (
    id SMALLINT PRIMARY KEY DEFAULT 1
        CHECK (id = 1),

    business_name VARCHAR(200) NOT NULL DEFAULT '',
    legal_name VARCHAR(200) NOT NULL DEFAULT '',
    business_type VARCHAR(100) NOT NULL DEFAULT 'Pharmacy / Medical Store',
    owner_name VARCHAR(150) NOT NULL DEFAULT '',

    phone VARCHAR(20) NOT NULL DEFAULT '',
    email VARCHAR(254) NOT NULL DEFAULT '',

    address_line1 VARCHAR(200) NOT NULL DEFAULT '',
    address_line2 VARCHAR(200) NOT NULL DEFAULT '',
    city VARCHAR(100) NOT NULL DEFAULT '',
    district VARCHAR(100) NOT NULL DEFAULT '',
    state VARCHAR(100) NOT NULL DEFAULT 'Rajasthan',
    pincode VARCHAR(6) NOT NULL DEFAULT '',
    country VARCHAR(100) NOT NULL DEFAULT 'India',

    gstin VARCHAR(15) NOT NULL DEFAULT '',
    drug_license_retail VARCHAR(100) NOT NULL DEFAULT '',
    drug_license_wholesale VARCHAR(100) NOT NULL DEFAULT '',
    pharmacy_registration VARCHAR(100) NOT NULL DEFAULT '',

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_business_profile_pincode
        CHECK (pincode = '' OR pincode ~ '^[0-9]{6}$'),

    CONSTRAINT chk_business_profile_gstin
        CHECK (gstin = '' OR gstin ~ '^[0-9A-Z]{15}$')
);

COMMIT;