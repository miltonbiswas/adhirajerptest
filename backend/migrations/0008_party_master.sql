BEGIN;

-- Common party master
CREATE TABLE parties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    party_code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(250) NOT NULL,
    contact_person VARCHAR(150),

    phone VARCHAR(20),
    alternate_phone VARCHAR(20),
    email VARCHAR(254),

    gstin VARCHAR(15),
    pan VARCHAR(10),
    drug_license_number VARCHAR(100),

    billing_address TEXT,
    shipping_address TEXT,
    city VARCHAR(100),
    district VARCHAR(100),
    state VARCHAR(100) NOT NULL DEFAULT 'Rajasthan',
    pincode VARCHAR(6) NOT NULL DEFAULT '',
    country VARCHAR(100) NOT NULL DEFAULT 'India',

    opening_balance NUMERIC(14,2) NOT NULL DEFAULT 0,
    opening_balance_type VARCHAR(12) NOT NULL DEFAULT 'PAYABLE',

    credit_limit NUMERIC(14,2) NOT NULL DEFAULT 0,
    credit_days INTEGER NOT NULL DEFAULT 0,

    notes TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_parties_opening_balance_type
        CHECK (
            opening_balance_type IN (
                'PAYABLE',
                'RECEIVABLE',
                'ADVANCE'
            )
        ),

    CONSTRAINT chk_parties_credit_limit
        CHECK (credit_limit >= 0),

    CONSTRAINT chk_parties_credit_days
        CHECK (credit_days >= 0),

    CONSTRAINT chk_parties_pincode
        CHECK (pincode = '' OR pincode ~ '^[0-9]{6}$'),

    CONSTRAINT chk_parties_gstin
        CHECK (gstin IS NULL OR gstin = '' OR gstin ~ '^[0-9A-Z]{15}$'),

    CONSTRAINT chk_parties_pan
        CHECK (pan IS NULL OR pan = '' OR pan ~ '^[A-Z]{5}[0-9]{4}[A-Z]$')
);

-- A party can have one or both roles.
CREATE TABLE party_roles (
    party_id UUID NOT NULL
        REFERENCES parties(id) ON DELETE RESTRICT,

    role VARCHAR(10) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_party_roles PRIMARY KEY (party_id, role),

    CONSTRAINT chk_party_roles_role
        CHECK (role IN ('SUPPLIER', 'CUSTOMER'))
);

-- Search and filtering indexes.
CREATE INDEX idx_parties_name
    ON parties (name);

CREATE INDEX idx_parties_phone
    ON parties (phone)
    WHERE phone IS NOT NULL;

CREATE INDEX idx_parties_gstin
    ON parties (gstin)
    WHERE gstin IS NOT NULL AND gstin <> '';

CREATE INDEX idx_party_roles_role
    ON party_roles (role, party_id);

COMMIT;