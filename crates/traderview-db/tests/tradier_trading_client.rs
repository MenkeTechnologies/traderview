//! Tradier REST client — wiremock contract tests covering the JSON we
//! ship + the 4xx error mapping.

use rust_decimal::Decimal;
use std::str::FromStr;
use traderview_db::tradier_trading::{
    Duration_, EquitySide, OrderType, OtocoBracket, PlaceEquityOrder, TradierError, TradierTrading,
};
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

async fn server_with_creds() -> (MockServer, TradierTrading) {
    let server = MockServer::start().await;
    let client = TradierTrading::with_base(server.uri(), "TOK", "ACCT-1");
    (server, client)
}

#[tokio::test]
async fn place_equity_market_sends_expected_form_fields() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("POST"))
        .and(path("/accounts/ACCT-1/orders"))
        .and(header("Authorization", "Bearer TOK"))
        .and(body_string_contains("class=equity"))
        .and(body_string_contains("symbol=AAPL"))
        .and(body_string_contains("side=buy"))
        .and(body_string_contains("quantity=10"))
        .and(body_string_contains("type=market"))
        .and(body_string_contains("duration=day"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "order": {"id": 12345, "status": "ok", "partner_id": "X"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let req = PlaceEquityOrder::market("AAPL", EquitySide::Buy, dec("10"));
    let resp = client.place_equity_order(&req).await.expect("place ok");
    assert_eq!(resp.id, 12345);
    assert_eq!(resp.status, "ok");
}

#[tokio::test]
async fn place_equity_limit_includes_price_field() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("POST"))
        .and(path("/accounts/ACCT-1/orders"))
        .and(body_string_contains("type=limit"))
        .and(body_string_contains("price=180.50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "order": {"id": 999, "status": "ok"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceEquityOrder {
        symbol: "MSFT".into(),
        side: EquitySide::Buy,
        quantity: dec("5"),
        order_type: OrderType::Limit,
        duration: Duration_::Day,
        price: Some(dec("180.50")),
        stop: None,
        tag: None,
    };
    let resp = client.place_equity_order(&req).await.expect("limit ok");
    assert_eq!(resp.id, 999);
}

#[tokio::test]
async fn place_otoco_bracket_encodes_three_legs() {
    let (server, client) = server_with_creds().await;
    // Tradier's OTOCO encoding requires every leg's symbol/side/qty/type
    // to be sent with [N] suffix. Verify each indexed pair we set
    // appears in the form body.
    Mock::given(method("POST"))
        .and(path("/accounts/ACCT-1/orders"))
        .and(body_string_contains("class=otoco"))
        .and(body_string_contains("symbol%5B0%5D=AAPL")) // url-encoded [0]
        .and(body_string_contains("side%5B0%5D=buy"))
        .and(body_string_contains("type%5B0%5D=market"))
        .and(body_string_contains("type%5B1%5D=limit"))
        .and(body_string_contains("price%5B1%5D=190"))
        .and(body_string_contains("type%5B2%5D=stop"))
        .and(body_string_contains("stop%5B2%5D=170"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "order": {"id": 7, "status": "ok"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let br = OtocoBracket {
        symbol: "AAPL".into(),
        entry_side: EquitySide::Buy,
        exit_side: EquitySide::Sell,
        quantity: dec("10"),
        take_profit_price: dec("190"),
        stop_loss_price: dec("170"),
        duration: Duration_::Day,
        tag: None,
    };
    let resp = client.place_otoco_bracket(&br).await.expect("otoco ok");
    assert_eq!(resp.id, 7);
}

#[tokio::test]
async fn place_401_maps_to_auth_failed() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("POST"))
        .and(path("/accounts/ACCT-1/orders"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "fault": {"faultstring": "Invalid access token"}
        })))
        .mount(&server)
        .await;
    let req = PlaceEquityOrder::market("AAPL", EquitySide::Buy, dec("1"));
    let err = client.place_equity_order(&req).await.expect_err("must error");
    assert!(matches!(err, TradierError::AuthFailed), "got {err:?}");
}

#[tokio::test]
async fn place_403_with_buying_power_maps_to_insufficient_buying_power() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("POST"))
        .and(path("/accounts/ACCT-1/orders"))
        .respond_with(ResponseTemplate::new(403).set_body_string(
            "{\"error\": \"Insufficient buying power for this order\"}",
        ))
        .mount(&server)
        .await;
    let req = PlaceEquityOrder::market("AAPL", EquitySide::Buy, dec("1000000"));
    let err = client.place_equity_order(&req).await.expect_err("must error");
    assert!(
        matches!(err, TradierError::InsufficientBuyingPower),
        "got {err:?}"
    );
}

#[tokio::test]
async fn place_400_maps_to_invalid_request() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("POST"))
        .and(path("/accounts/ACCT-1/orders"))
        .respond_with(ResponseTemplate::new(400).set_body_string(
            "{\"errors\":{\"error\":[\"Order quantity must be greater than zero\"]}}",
        ))
        .mount(&server)
        .await;
    let req = PlaceEquityOrder::market("AAPL", EquitySide::Buy, dec("0"));
    let err = client.place_equity_order(&req).await.expect_err("must error");
    match err {
        TradierError::InvalidRequest(body) => {
            assert!(body.contains("greater than zero"), "body={body}");
        }
        other => panic!("expected InvalidRequest, got {other:?}"),
    }
}

#[tokio::test]
async fn cancel_order_204_returns_ok() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("DELETE"))
        .and(path("/accounts/ACCT-1/orders/12345"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    client.cancel_order(12345).await.expect("cancel ok");
}

#[tokio::test]
async fn get_balances_decodes() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("GET"))
        .and(path("/accounts/ACCT-1/balances"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "balances": {
                "account_number": "ACCT-1",
                "account_type": "margin",
                "total_equity": "100000.00",
                "total_cash": "25000.00",
                "stock_buying_power": "50000.00",
                "option_buying_power": "25000.00"
            }
        })))
        .mount(&server)
        .await;
    let b = client.get_balances().await.expect("balances");
    assert_eq!(b.balances.total_equity, Some(dec("100000.00")));
    assert_eq!(b.balances.stock_buying_power, Some(dec("50000.00")));
}

#[tokio::test]
async fn get_positions_handles_empty_string() {
    // Tradier returns "positions": "null" (a literal string) when the
    // account is empty. The untagged enum lets us decode that shape
    // without panicking.
    let (server, client) = server_with_creds().await;
    Mock::given(method("GET"))
        .and(path("/accounts/ACCT-1/positions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "positions": "null"
        })))
        .mount(&server)
        .await;
    let p = client.get_positions().await.expect("positions");
    assert!(p.positions.is_some());
}

#[tokio::test]
async fn get_positions_handles_single_position() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("GET"))
        .and(path("/accounts/ACCT-1/positions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "positions": {
                "position": {
                    "cost_basis": "1800.00",
                    "quantity": "10",
                    "symbol": "AAPL"
                }
            }
        })))
        .mount(&server)
        .await;
    let p = client.get_positions().await.expect("positions");
    assert!(p.positions.is_some());
}
