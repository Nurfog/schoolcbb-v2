use crate::payment_gateway::{MockGateway, PaymentGateway, PaymentGatewayConfig, PaymentInitRequest};

#[test]
fn test_from_env_returns_none_when_unset() {
    let prev = std::env::var("PAYMENT_PROVIDER").ok();
    unsafe { std::env::remove_var("PAYMENT_PROVIDER") };
    let cfg = PaymentGatewayConfig::from_env();
    assert!(cfg.is_none());
    if let Some(val) = prev {
        unsafe { std::env::set_var("PAYMENT_PROVIDER", val) };
    }
}

#[test]
fn test_mock_gateway_confirm_returns_success() {
    let gateway = MockGateway;
    let result = gateway.confirm_transaction("mock_token");
    assert!(result.is_ok());
    let payment = result.unwrap();
    assert!(payment.success);
    assert_eq!(payment.authorization_code, "MOCK001");
    assert_eq!(payment.payment_type, "MOCK");
}

#[test]
fn test_mock_gateway_init_returns_url_and_token() {
    let gateway = MockGateway;
    let req = PaymentInitRequest {
        amount: 100.0,
        description: "Test".into(),
        student_id: "stu-1".into(),
        fee_id: "fee-1".into(),
        payer_email: None,
    };
    let result = gateway.init_transaction(&req);
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert!(resp.url.contains("mock=true"));
    assert!(resp.token.starts_with("mock_"));
}
