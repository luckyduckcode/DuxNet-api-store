-- Fix schema to match the models

-- Drop existing tables in correct order (children first)
DROP TABLE IF EXISTS escrow_contracts CASCADE;
DROP TABLE IF EXISTS mining_sessions CASCADE;
DROP TABLE IF EXISTS wallet_balances CASCADE;
DROP TABLE IF EXISTS analytics CASCADE;
DROP TABLE IF EXISTS reputation_scores CASCADE;
DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS services CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- Create tables with correct schema to match models

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR NOT NULL UNIQUE,
    display_name VARCHAR,
    email VARCHAR UNIQUE,
    wallet_address VARCHAR,
    public_key VARCHAR,
    reputation_score DECIMAL DEFAULT 0.0,
    total_earnings BIGINT DEFAULT 0,
    total_spent BIGINT DEFAULT 0,
    service_count INTEGER DEFAULT 0,
    rating DECIMAL DEFAULT 0.0,
    metadata JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Services table  
CREATE TABLE services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    description TEXT,
    manifest JSONB NOT NULL,
    service_type VARCHAR NOT NULL,
    version VARCHAR NOT NULL DEFAULT '1.0.0',
    status VARCHAR NOT NULL DEFAULT 'pending',
    tags TEXT[] DEFAULT '{}',
    pricing JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_hash VARCHAR NOT NULL UNIQUE,
    from_user_id UUID REFERENCES users(id),
    to_user_id UUID REFERENCES users(id),
    from_address VARCHAR,
    to_address VARCHAR,
    amount BIGINT NOT NULL,
    currency VARCHAR NOT NULL DEFAULT 'DUX',
    transaction_type VARCHAR NOT NULL,
    status VARCHAR NOT NULL DEFAULT 'pending',
    confirmations INTEGER DEFAULT 0,
    block_hash VARCHAR,
    block_height BIGINT,
    fee BIGINT DEFAULT 0,
    gas_used BIGINT DEFAULT 0,
    data JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    confirmed_at TIMESTAMPTZ
);

-- Reputation scores table
CREATE TABLE reputation_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    score DECIMAL NOT NULL DEFAULT 0.0,
    total_ratings INTEGER DEFAULT 0,
    category VARCHAR,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, category)
);

-- Analytics table
CREATE TABLE analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name VARCHAR NOT NULL,
    metric_value DECIMAL NOT NULL,
    labels JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB DEFAULT '{}'
);

-- Wallet balances table
CREATE TABLE wallet_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    currency VARCHAR NOT NULL,
    balance BIGINT NOT NULL DEFAULT 0,
    pending_balance BIGINT DEFAULT 0,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, currency)
);

-- Mining sessions table
CREATE TABLE mining_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMPTZ,
    hash_rate DECIMAL DEFAULT 0.0,
    shares_submitted INTEGER DEFAULT 0,
    shares_accepted INTEGER DEFAULT 0,
    reward BIGINT DEFAULT 0,
    status VARCHAR DEFAULT 'active',
    metadata JSONB DEFAULT '{}'
);

-- Escrow contracts table
CREATE TABLE escrow_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    buyer_id UUID NOT NULL REFERENCES users(id),
    seller_id UUID NOT NULL REFERENCES users(id),
    service_id UUID REFERENCES services(id),
    amount BIGINT NOT NULL,
    currency VARCHAR NOT NULL DEFAULT 'DUX',
    status VARCHAR NOT NULL DEFAULT 'pending',
    conditions JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    released_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'
);

-- Create indexes for better performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_wallet_address ON users(wallet_address);
CREATE INDEX idx_services_owner_id ON services(owner_id);
CREATE INDEX idx_services_status ON services(status);
CREATE INDEX idx_services_service_type ON services(service_type);
CREATE INDEX idx_transactions_hash ON transactions(transaction_hash);
CREATE INDEX idx_transactions_from_user ON transactions(from_user_id);
CREATE INDEX idx_transactions_to_user ON transactions(to_user_id);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_reputation_user_id ON reputation_scores(user_id);
CREATE INDEX idx_analytics_metric ON analytics(metric_name);
CREATE INDEX idx_analytics_timestamp ON analytics(timestamp);
CREATE INDEX idx_wallet_user_currency ON wallet_balances(user_id, currency);
CREATE INDEX idx_mining_user_id ON mining_sessions(user_id);
CREATE INDEX idx_escrow_buyer ON escrow_contracts(buyer_id);
CREATE INDEX idx_escrow_seller ON escrow_contracts(seller_id);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_services_updated_at BEFORE UPDATE ON services FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_reputation_updated_at BEFORE UPDATE ON reputation_scores FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
