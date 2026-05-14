-- CRM Schema for Sales Pipeline

CREATE TABLE IF NOT EXISTS crm_sales_stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    is_final BOOLEAN NOT NULL DEFAULT false,
    color VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_sales_prospects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    company VARCHAR(255),
    position VARCHAR(255),
    source VARCHAR(50),
    requirements JSONB DEFAULT '{}',
    current_stage_id UUID REFERENCES crm_sales_stages(id),
    assigned_to UUID,
    estimated_value DECIMAL(12,2),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_sales_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prospect_id UUID NOT NULL REFERENCES crm_sales_prospects(id) ON DELETE CASCADE,
    activity_type VARCHAR(50) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    description TEXT,
    scheduled_at TIMESTAMPTZ,
    is_completed BOOLEAN NOT NULL DEFAULT false,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_sales_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prospect_id UUID NOT NULL REFERENCES crm_sales_prospects(id) ON DELETE CASCADE,
    plan_id UUID,
    modules JSONB DEFAULT '{}',
    total_value DECIMAL(12,2) NOT NULL DEFAULT 0,
    discount DECIMAL(12,2) NOT NULL DEFAULT 0,
    version INT NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    notes TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_sales_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prospect_id UUID NOT NULL REFERENCES crm_sales_prospects(id) ON DELETE CASCADE,
    plan_id UUID,
    modules JSONB DEFAULT '{}',
    total_value DECIMAL(12,2) NOT NULL DEFAULT 0,
    discount DECIMAL(12,2) NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    signed_at TIMESTAMPTZ,
    verified_at TIMESTAMPTZ,
    activated_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_sales_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id UUID NOT NULL REFERENCES crm_sales_contracts(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_url TEXT,
    doc_type VARCHAR(50) NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    uploaded_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default pipeline stages
INSERT INTO crm_sales_stages (id, name, sort_order, is_final, color) VALUES
    ('10000000-0000-0000-0000-000000000001', 'Nuevo', 1, false, '#6B7280'),
    ('10000000-0000-0000-0000-000000000002', 'Contactado', 2, false, '#3B82F6'),
    ('10000000-0000-0000-0000-000000000003', 'Requisitos', 3, false, '#8B5CF6'),
    ('10000000-0000-0000-0000-000000000004', 'Propuesta', 4, false, '#F59E0B'),
    ('10000000-0000-0000-0000-000000000005', 'Negociación', 5, false, '#F97316'),
    ('10000000-0000-0000-0000-000000000006', 'Contrato', 6, false, '#10B981'),
    ('10000000-0000-0000-0000-000000000007', 'Cerrado Ganado', 7, true, '#059669'),
    ('10000000-0000-0000-0000-000000000008', 'Cerrado Perdido', 8, true, '#EF4444')
ON CONFLICT DO NOTHING;
