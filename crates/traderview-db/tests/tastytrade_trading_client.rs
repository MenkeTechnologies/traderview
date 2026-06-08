//! Tastytrade REST client — wiremock contract tests.

use rust_decimal::Decimal;
use std::str::FromStr;
use traderview_db::tastytrade_trading::{
    Auth, EquityAction, OrderType, PlaceEquityOrder, PriceEffect, TastytradeError,
    TastytradeTrading, TimeInForce,
};
use wiremock::matchers::{body_json_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

async fn server_with_token() -> (MockServer, TastytradeTrading) {
    let server = MockServer::start().await;
    let client = TastytradeTrading::with_base(
        server.uri(),
        Auth::SessionToken("PRE-MINTED-TOKEN".into()),
        "5WX12345",
    );
    (server, client)
}

#[tokio::test]
async fn place_equity_market_sends_expected_json() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/accounts/5WX12345/orders"))
        .and(header("Authorization", "PRE-MINTED-TOKEN"))
        .and(header("Accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "order": {
                    "id": 987654,
                    "status": "Routed",
                    "underlying-symbol": "AAPL"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceEquityOrder::market("AAPL", EquityAction::BuyToOpen, dec("10"));
    let resp = client.place_equity_order(&req).await.expect("place ok");
    assert_eq!(resp.id, 987654);
    assert_eq!(resp.status, "Routed");
    assert_eq!(resp.underlying_symbol.as_deref(), Some("AAPL"));
}

#[tokio::test]
async fn place_equity_limit_includes_price_and_price_effect() {
    let (server, client) = server_with_token().await;
    // Verify the exact JSON we ship — including the hyphenated field
    // names Tastytrade requires (order-type, time-in-force, price-effect,
    // instrument-type). One drift in field naming and they 422.
    Mock::given(method("POST"))
        .and(path("/accounts/5WX12345/orders"))
        .and(body_json_string(
            serde_json::to_string(&serde_json::json!({
                "order-type": "Limit",
                "time-in-force": "Day",
                "price": "180.50",
                "price-effect": "Debit",
                "legs": [
                    {
                        "symbol": "MSFT",
                        "instrument-type": "Equity",
                        "action": "Buy to Open",
                        "quantity": "5"
                    }
                ]
            }))
            .unwrap(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"order": {"id": 1, "status": "Routed"}}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceEquityOrder {
        symbol: "MSFT".into(),
        action: EquityAction::BuyToOpen,
        quantity: dec("5"),
        order_type: OrderType::Limit,
        time_in_force: TimeInForce::Day,
        price: Some(dec("180.50")),
        price_effect: Some(PriceEffect::Debit),
        stop_trigger: None,
    };
    let _ = client.place_equity_order(&req).await.expect("limit ok");
}

#[tokio::test]
async fn place_401_maps_to_auth_failed() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/accounts/5WX12345/orders"))
        .respond_with(ResponseTemplate::new(401).set_body_string("{\"error\": \"invalid token\"}"))
        .mount(&server)
        .await;
    let req = PlaceEquityOrder::market("AAPL", EquityAction::BuyToOpen, dec("1"));
    let err = client
        .place_equity_order(&req)
        .await
        .expect_err("must error");
    assert!(matches!(err, TastytradeError::AuthFailed), "got {err:?}");
}

#[tokio::test]
async fn place_403_with_buying_power_maps_to_insufficient_buying_power() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/accounts/5WX12345/orders"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_string("{\"error\": \"Insufficient buying power for this order\"}"),
        )
        .mount(&server)
        .await;
    let req = PlaceEquityOrder::market("AAPL", EquityAction::BuyToOpen, dec("99999999"));
    let err = client
        .place_equity_order(&req)
        .await
        .expect_err("must error");
    assert!(
        matches!(err, TastytradeError::InsufficientBuyingPower),
        "got {err:?}"
    );
}

#[tokio::test]
async fn place_422_maps_to_invalid_request() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/accounts/5WX12345/orders"))
        .respond_with(ResponseTemplate::new(422).set_body_string(
            "{\"error\": {\"code\": \"validation_error\", \"message\": \"quantity must be positive\"}}",
        ))
        .mount(&server)
        .await;
    let req = PlaceEquityOrder::market("AAPL", EquityAction::BuyToOpen, dec("0"));
    let err = client
        .place_equity_order(&req)
        .await
        .expect_err("must error");
    match err {
        TastytradeError::InvalidRequest(body) => {
            assert!(body.contains("must be positive"), "body={body}");
        }
        other => panic!("expected InvalidRequest, got {other:?}"),
    }
}

#[tokio::test]
async fn user_pass_auth_logs_in_then_uses_token() {
    let server = MockServer::start().await;
    // First call: POST /sessions with login+password+remember-me.
    Mock::given(method("POST"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "session-token": "fresh-session-abc123",
                "user": {"email": "x@y.z"}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    // Two follow-up calls: both with the same fresh-session token.
    // expect(2) verifies the second call reuses the token and DOES NOT
    // re-login (the /sessions mock has expect(1), so any second login
    // would fail the test there).
    Mock::given(method("POST"))
        .and(path("/accounts/5WX12345/orders"))
        .and(header("Authorization", "fresh-session-abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"order": {"id": 42, "status": "Routed"}}
        })))
        .expect(2)
        .mount(&server)
        .await;
    let client = TastytradeTrading::with_base(
        server.uri(),
        Auth::UserPass {
            login: "a@b.com".into(),
            password: "pw".into(),
            remember_me: true,
        },
        "5WX12345",
    );
    let req = PlaceEquityOrder::market("AAPL", EquityAction::BuyToOpen, dec("1"));
    let resp = client.place_equity_order(&req).await.expect("place ok");
    assert_eq!(resp.id, 42);

    // Verify the token gets reused — a 2nd order should not trigger another login.
    let _ = client.place_equity_order(&req).await.expect("place ok #2");
}

#[tokio::test]
async fn cancel_order_204_returns_ok() {
    let (server, client) = server_with_token().await;
    Mock::given(method("DELETE"))
        .and(path("/accounts/5WX12345/orders/987654"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    client.cancel_order(987654).await.expect("cancel ok");
}

#[tokio::test]
async fn get_balances_decodes_hyphenated_fields() {
    let (server, client) = server_with_token().await;
    Mock::given(method("GET"))
        .and(path("/accounts/5WX12345/balances"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "account-number": "5WX12345",
                "net-liquidating-value": "100000.00",
                "cash-balance": "25000.00",
                "equity-buying-power": "50000.00"
            }
        })))
        .mount(&server)
        .await;
    let b = client.get_balances().await.expect("balances");
    assert_eq!(b.data.account_number.as_deref(), Some("5WX12345"));
    assert_eq!(b.data.net_liquidating_value, Some(dec("100000.00")));
    assert_eq!(b.data.equity_buying_power, Some(dec("50000.00")));
}
