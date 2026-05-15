-- CRM Extensions: Sales Team, Round-Robin, CSV Import, Invoice, Tax support

-- Sales Team (agents with goals)
CREATE TABLE IF NOT EXISTS crm_sales_agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE,
    quota_monthly DECIMAL(12,2) DEFAULT 0,
    quota_quarterly DECIMAL(12,2) DEFAULT 0,
    commission_rate DECIMAL(5,2) DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_sales_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES crm_sales_agents(id) ON DELETE CASCADE,
    goal_type VARCHAR(50) NOT NULL, -- 'monthly', 'quarterly', 'yearly'
    target_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    target_count INT NOT NULL DEFAULT 0,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    achieved_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    achieved_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Round-robin configuration
CREATE TABLE IF NOT EXISTS crm_round_robin_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    active BOOLEAN NOT NULL DEFAULT false,
    last_assigned_index INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- CSV Import tracking
CREATE TABLE IF NOT EXISTS crm_sales_imports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_name VARCHAR(255) NOT NULL,
    total_rows INT NOT NULL DEFAULT 0,
    imported_rows INT NOT NULL DEFAULT 0,
    failed_rows INT NOT NULL DEFAULT 0,
    errors JSONB DEFAULT '[]',
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, processing, completed, failed
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add tax fields to contracts
ALTER TABLE crm_sales_contracts ADD COLUMN IF NOT EXISTS tax_id VARCHAR(50);
ALTER TABLE crm_sales_contracts ADD COLUMN IF NOT EXISTS tax_rate DECIMAL(5,2) DEFAULT 19.00;
ALTER TABLE crm_sales_contracts ADD COLUMN IF NOT EXISTS subtotal DECIMAL(12,2);
ALTER TABLE crm_sales_contracts ADD COLUMN IF NOT EXISTS tax_amount DECIMAL(12,2);
ALTER TABLE crm_sales_contracts ADD COLUMN IF NOT EXISTS invoices JSONB DEFAULT '[]';

-- Insert default round-robin config
INSERT INTO crm_round_robin_config (id, active) VALUES ('00000000-0000-0000-0000-000000000001', false)
ON CONFLICT (id) DO NOTHING;
