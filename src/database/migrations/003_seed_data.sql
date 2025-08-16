-- Seed data for testing the DuxNet database

-- Insert test users
INSERT INTO users (id, username, display_name, email, wallet_address, public_key, metadata, is_active) VALUES
(
    '550e8400-e29b-41d4-a716-446655440000',
    'alice_dev',
    'Alice Developer',
    'alice@duxnet.example',
    '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
    'ed25519:32bytes_public_key_placeholder_alice_dev',
    '{"role": "developer", "verified": true}',
    true
),
(
    '550e8400-e29b-41d4-a716-446655440001', 
    'bob_provider',
    'Bob Service Provider',
    'bob@duxnet.example',
    '1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2',
    'ed25519:32bytes_public_key_placeholder_bob_provider',
    '{"role": "service_provider", "verified": true}',
    true
),
(
    '550e8400-e29b-41d4-a716-446655440002',
    'charlie_user',
    'Charlie End User', 
    'charlie@duxnet.example',
    '1JArS6jzE3AJ9sZ3aXPzGRqhwYjFQjxSJ7',
    'ed25519:32bytes_public_key_placeholder_charlie_user',
    '{"role": "end_user", "verified": false}',
    true
);

-- Insert test services
INSERT INTO services (id, name, description, endpoint, manifest, provider_id, is_active, metadata) VALUES
(
    '660e8400-e29b-41d4-a716-446655440000',
    'ai-text-analyzer',
    'Advanced AI-powered text analysis service',
    'https://ai-analyzer.duxnet.example/api',
    '{"version": "1.0.0", "cpu": 100, "memory": 512, "disk": 1024, "network": true}',
    '550e8400-e29b-41d4-a716-446655440000',
    true,
    '{"category": "ai", "pricing": "per_request", "rate_limit": 1000}'
),
(
    '660e8400-e29b-41d4-a716-446655440001',
    'data-processor',
    'High-performance data processing pipeline',
    'https://data-processor.duxnet.example/api',
    '{"version": "2.1.0", "cpu": 200, "memory": 1024, "disk": 2048, "network": true}',
    '550e8400-e29b-41d4-a716-446655440001',
    true,
    '{"category": "data", "pricing": "per_hour", "min_duration": 3600}'
);

-- Insert test transactions
INSERT INTO transactions (id, transaction_type, from_user_id, to_user_id, service_id, amount, dux_amount, status, metadata) VALUES
(
    '770e8400-e29b-41d4-a716-446655440000',
    'service_payment',
    '550e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440000',
    '660e8400-e29b-41d4-a716-446655440000',
    1000,
    1000,
    'completed',
    '{"processing_time": 1.5, "data_size": 1024}'
),
(
    '770e8400-e29b-41d4-a716-446655440001',
    'service_payment',
    '550e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001',
    5000,
    5000,
    'pending',
    '{"estimated_duration": 7200, "job_priority": "high"}'
);

-- Insert test reputation scores
INSERT INTO reputation_scores (id, user_id, service_id, score, review, reviewer_id) VALUES
(
    '880e8400-e29b-41d4-a716-446655440000',
    '550e8400-e29b-41d4-a716-446655440000',
    '660e8400-e29b-41d4-a716-446655440000',
    4.8,
    'Excellent AI analysis service, very fast and accurate results',
    '550e8400-e29b-41d4-a716-446655440002'
),
(
    '880e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001',
    4.5,
    'Good data processing service, could be faster but reliable',
    '550e8400-e29b-41d4-a716-446655440002'
);

-- Insert test analytics events
INSERT INTO analytics (id, event_type, user_id, service_id, metadata) VALUES
(
    '990e8400-e29b-41d4-a716-446655440000',
    'service_request',
    '550e8400-e29b-41d4-a716-446655440002',
    '660e8400-e29b-41d4-a716-446655440000',
    '{"request_size": 1024, "response_time": 1.5, "success": true}'
),
(
    '990e8400-e29b-41d4-a716-446655440001',
    'user_login',
    '550e8400-e29b-41d4-a716-446655440002',
    null,
    '{"ip_address": "192.168.1.100", "user_agent": "DuxNet-Client/1.0"}'
);

-- Insert test wallet balances
INSERT INTO wallet_balances (id, user_id, currency, balance, locked_balance) VALUES
(
    'aa0e8400-e29b-41d4-a716-446655440000',
    '550e8400-e29b-41d4-a716-446655440000',
    'DUX',
    50000,
    0
),
(
    'aa0e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'DUX',
    75000,
    5000
),
(
    'aa0e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440002',
    'DUX',
    25000,
    1000
);

-- Insert test mining sessions
INSERT INTO mining_sessions (id, user_id, status, hash_rate, start_time, end_time, blocks_mined, rewards_earned) VALUES
(
    'bb0e8400-e29b-41d4-a716-446655440000',
    '550e8400-e29b-41d4-a716-446655440000',
    'completed',
    1250.5,
    '2024-08-01 10:00:00',
    '2024-08-01 11:30:00',
    3,
    750
),
(
    'bb0e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'active',
    980.2,
    '2024-08-16 09:00:00',
    null,
    1,
    250
);

-- Insert test escrow contracts
INSERT INTO escrow_contracts (id, service_id, buyer_id, seller_id, amount, status, release_conditions) VALUES
(
    'cc0e8400-e29b-41d4-a716-446655440000',
    '660e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440001',
    5000,
    'active',
    '{"completion_criteria": "data_processing_complete", "timeout_hours": 24}'
);
