-- Add RUT support for Prospects and Contracts
ALTER TABLE crm_sales_prospects ADD COLUMN IF NOT EXISTS rut VARCHAR(12);
ALTER TABLE crm_sales_contracts ADD COLUMN IF NOT EXISTS tax_id VARCHAR(12); -- RUT for the contract entity
