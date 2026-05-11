use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentGatewayConfig {
    pub provider: String,
    pub webpay_commerce_code: String,
    pub webpay_api_key: String,
    pub webpay_environment: String,
    pub return_url: String,
}

impl PaymentGatewayConfig {
    pub fn from_env() -> Option<Self> {
        let provider = std::env::var("PAYMENT_PROVIDER").unwrap_or_default();
        if provider.is_empty() {
            return None;
        }
        Some(Self {
            provider,
            webpay_commerce_code: std::env::var("WEBPAY_COMMERCE_CODE").unwrap_or_default(),
            webpay_api_key: std::env::var("WEBPAY_API_KEY").unwrap_or_default(),
            webpay_environment: std::env::var("WEBPAY_ENVIRONMENT")
                .unwrap_or_else(|_| "integration".into()),
            return_url: std::env::var("PAYMENT_RETURN_URL")
                .unwrap_or_else(|_| "http://localhost:8080/finance".into()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInitRequest {
    pub amount: f64,
    pub description: String,
    pub student_id: String,
    pub fee_id: String,
    pub payer_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInitResponse {
    pub url: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub success: bool,
    pub amount: f64,
    pub transaction_id: String,
    pub authorization_code: String,
    pub payment_type: String,
}

pub trait PaymentGateway: Send + Sync {
    fn provider_name(&self) -> &str;
    fn init_transaction(&self, req: &PaymentInitRequest) -> Result<PaymentInitResponse, String>;
    fn confirm_transaction(&self, token: &str) -> Result<PaymentResult, String>;
}

pub struct WebpayGateway {
    pub config: PaymentGatewayConfig,
}

impl PaymentGateway for WebpayGateway {
    fn provider_name(&self) -> &str {
        "Webpay Plus"
    }

    fn init_transaction(&self, req: &PaymentInitRequest) -> Result<PaymentInitResponse, String> {
        let amount_cents = (req.amount * 100.0).round() as i64;
        let buy_order = format!("{}-{}", req.fee_id, chrono::Utc::now().timestamp());

        let session_id = req.student_id.clone();

        let payload = serde_json::json!({
            "buy_order": buy_order,
            "session_id": session_id,
            "amount": amount_cents,
            "return_url": self.config.return_url,
        });

        let client = reqwest::blocking::Client::new();
        let env = &self.config.webpay_environment;
        let url = if env == "production" {
            "https://webpay3g.transbank.cl/rswebpaytransaction/api/webpay/v1.0/transactions"
        } else {
            "https://webpay3gint.transbank.cl/rswebpaytransaction/api/webpay/v1.0/transactions"
        };

        let resp = client
            .post(url)
            .header("Tbk-Api-Key-Secret", &self.config.webpay_api_key)
            .header("Tbk-Api-Key-Id", &self.config.webpay_commerce_code)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .map_err(|e| format!("Error de conexión con Webpay: {e}"))?;

        let data: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Error al parsear respuesta Webpay: {e}"))?;

        let url = data["url"]
            .as_str()
            .ok_or_else(|| "Webpay no retornó URL".to_string())?
            .to_string();
        let token = data["token"]
            .as_str()
            .ok_or_else(|| "Webpay no retornó token".to_string())?
            .to_string();

        Ok(PaymentInitResponse { url, token })
    }

    fn confirm_transaction(&self, token: &str) -> Result<PaymentResult, String> {
        let client = reqwest::blocking::Client::new();
        let env = &self.config.webpay_environment;
        let url = if env == "production" {
            format!(
                "https://webpay3g.transbank.cl/rswebpaytransaction/api/webpay/v1.0/transactions/{token}"
            )
        } else {
            format!(
                "https://webpay3gint.transbank.cl/rswebpaytransaction/api/webpay/v1.0/transactions/{token}"
            )
        };

        let resp = client
            .put(url)
            .header("Tbk-Api-Key-Secret", &self.config.webpay_api_key)
            .header("Tbk-Api-Key-Id", &self.config.webpay_commerce_code)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|e| format!("Error al confirmar transacción Webpay: {e}"))?;

        let data: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Error al parsear confirmación Webpay: {e}"))?;

        let status = data["status"].as_str().unwrap_or("FAILED");
        let success = status == "AUTHORIZED";

        Ok(PaymentResult {
            success,
            amount: data["amount"].as_f64().unwrap_or(0.0) / 100.0,
            transaction_id: data["buy_order"].as_str().unwrap_or("").to_string(),
            authorization_code: data["authorization_code"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            payment_type: data["payment_type_code"].as_str().unwrap_or("").to_string(),
        })
    }
}

pub struct MockGateway;

impl PaymentGateway for MockGateway {
    fn provider_name(&self) -> &str {
        "Mock (desarrollo)"
    }

    fn init_transaction(&self, req: &PaymentInitRequest) -> Result<PaymentInitResponse, String> {
        let token = format!("mock_{}_{}", req.fee_id, chrono::Utc::now().timestamp());
        let url = format!("/api/finance/payment/return?token_ws={}&mock=true", token);
        Ok(PaymentInitResponse { url, token })
    }

    fn confirm_transaction(&self, _token: &str) -> Result<PaymentResult, String> {
        Ok(PaymentResult {
            success: true,
            amount: 0.0,
            transaction_id: format!("tx_mock_{}", chrono::Utc::now().timestamp()),
            authorization_code: "MOCK001".to_string(),
            payment_type: "MOCK".to_string(),
        })
    }
}
