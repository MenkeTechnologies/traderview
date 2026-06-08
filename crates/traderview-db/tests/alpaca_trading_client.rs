//! Alpaca trading REST surface — wiremock-driven contract tests.
//!
//! Covers what would actually break against a live broker: the JSON
//! request shape we ship, the error mapping for the 4xx responses
//! Alpaca actually returns, and the cancel endpoint's 204 path.

use rust_decimal::Decimal;
use std::str::FromStr;
use traderview_db::alpaca_trading::{
    AlpacaError, AlpacaTrading, PlaceOrderRequest,
};
use uuid::Uuid;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

async fn server_with_creds() -> (MockServer, AlpacaTrading) {
    let server = MockServer::start().await;
    let client = AlpacaTrading::with_rest_base(server.uri(), "KEY", "SECRET");
    (server, client)
}

#[tokio::test]
async fn place_simple_market_sends_expected_payload() {
    let (server, client) = server_with_creds().await;
    let coid = Uuid::new_v4();

    Mock::given(method("POST"))
        .and(path("/v2/orders"))
        .and(header("APCA-API-KEY-ID", "KEY"))
        .and(header("APCA-API-SECRET-KEY", "SECRET"))
        .and(body_json(serde_json::json!({
            "symbol": "AAPL",
            "qty": "10",
            "side": "buy",
            "type": "market",
            "time_in_force": "day",
            "client_order_id": coid.to_string(),
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "broker-id-1",
            "client_order_id": coid.to_string(),
            "status": "accepted",
            "symbol": "AAPL",
            "side": "buy",
            "type": "market",
            "qty": "10",
            "filled_qty": "0",
            "filled_avg_price": null,
            "limit_price": null,
            "stop_price": null,
            "time_in_force": "day",
            "order_class": "simple",
            "created_at": "2026-06-07T13:30:00Z",
            "updated_at": "2026-06-07T13:30:00Z",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let req = PlaceOrderRequest::market("AAPL", "buy", dec("10"), coid);
    let resp = client.place_order(&req).await.expect("place ok");
    assert_eq!(resp.id, "broker-id-1");
    assert_eq!(resp.status, "accepted");
}

#[tokio::test]
async fn place_bracket_market_includes_legs_and_class() {
    let (server, client) = server_with_creds().await;
    let coid = Uuid::new_v4();

    Mock::given(method("POST"))
        .and(path("/v2/orders"))
        .and(body_json(serde_json::json!({
            "symbol": "TSLA",
            "qty": "5",
            "side": "buy",
            "type": "market",
            "time_in_force": "day",
            "client_order_id": coid.to_string(),
            "order_class": "bracket",
            "take_profit": {"limit_price": "260"},
            "stop_loss": {"stop_price": "240"},
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "br-1",
            "client_order_id": coid.to_string(),
            "status": "accepted",
            "symbol": "TSLA",
            "side": "buy",
            "type": "market",
            "qty": "5",
            "filled_qty": "0",
            "filled_avg_price": null,
            "limit_price": null,
            "stop_price": null,
            "time_in_force": "day",
            "order_class": "bracket",
            "created_at": null,
            "updated_at": null,
        })))
        .expect(1)
        .mount(&server)
        .await;

    let req = PlaceOrderRequest::bracket_market(
        "TSLA", "buy", dec("5"), coid, dec("260"), dec("240"),
    );
    let resp = client.place_order(&req).await.expect("bracket ok");
    assert_eq!(resp.order_class.as_deref(), Some("bracket"));
}

#[tokio::test]
async fn place_403_with_buying_power_message_maps_to_insufficient_buying_power() {
    let (server, client) = server_with_creds().await;

    Mock::given(method("POST"))
        .and(path("/v2/orders"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "code": 40310000,
            "message": "insufficient buying power",
        })))
        .mount(&server)
        .await;

    let req = PlaceOrderRequest::market("MSFT", "buy", dec("100000"), Uuid::new_v4());
    let err = client.place_order(&req).await.expect_err("must error");
    assert!(matches!(err, AlpacaError::InsufficientBuyingPower), "got {err:?}");
}

#[tokio::test]
async fn place_403_auth_maps_to_auth_failed() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("POST"))
        .and(path("/v2/orders"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "code": 40110000,
            "message": "authentication failed",
        })))
        .mount(&server)
        .await;
    let req = PlaceOrderRequest::market("AAPL", "buy", dec("1"), Uuid::new_v4());
    let err = client.place_order(&req).await.expect_err("must error");
    assert!(matches!(err, AlpacaError::AuthFailed), "got {err:?}");
}

#[tokio::test]
async fn place_422_maps_to_invalid_request_with_body() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("POST"))
        .and(path("/v2/orders"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "code": 42210000,
            "message": "limit_price has too many decimal places",
        })))
        .mount(&server)
        .await;
    let mut req = PlaceOrderRequest::market("AAPL", "buy", dec("1"), Uuid::new_v4());
    req.order_type = "limit".into();
    req.limit_price = Some(dec("180.12345"));
    let err = client.place_order(&req).await.expect_err("must error");
    match err {
        AlpacaError::InvalidRequest(body) => {
            assert!(body.contains("too many decimal places"), "body={body}");
        }
        other => panic!("expected InvalidRequest, got {other:?}"),
    }
}

#[tokio::test]
async fn cancel_204_returns_ok() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("DELETE"))
        .and(path("/v2/orders/broker-id-1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    client.cancel_order("broker-id-1").await.expect("cancel ok");
}

#[tokio::test]
async fn cancel_422_after_terminal_state_maps_to_invalid_request() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("DELETE"))
        .and(path("/v2/orders/already-filled"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "message": "order is already in a terminal state",
        })))
        .mount(&server)
        .await;
    let err = client.cancel_order("already-filled").await.expect_err("must error");
    assert!(matches!(err, AlpacaError::InvalidRequest(_)), "got {err:?}");
}

#[tokio::test]
async fn get_order_by_client_id_round_trips() {
    let (server, client) = server_with_creds().await;
    let coid = Uuid::new_v4();
    Mock::given(method("GET"))
        .and(path("/v2/orders:by_client_order_id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "broker-77",
            "client_order_id": coid.to_string(),
            "status": "filled",
            "symbol": "AAPL",
            "side": "buy",
            "type": "market",
            "qty": "10",
            "filled_qty": "10",
            "filled_avg_price": "180.50",
            "limit_price": null,
            "stop_price": null,
            "time_in_force": "day",
            "order_class": "simple",
            "created_at": null,
            "updated_at": null,
        })))
        .expect(1)
        .mount(&server)
        .await;
    let resp = client.get_order_by_client_id(coid).await.expect("get ok");
    assert_eq!(resp.filled_avg_price, Some(dec("180.50")));
}

#[tokio::test]
async fn list_positions_decodes_array() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("GET"))
        .and(path("/v2/positions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "symbol": "AAPL",
                "qty": "10",
                "side": "long",
                "avg_entry_price": "180.00",
                "market_value": "1810.00",
                "cost_basis": "1800.00",
                "unrealized_pl": "10.00",
                "current_price": "181.00"
            }
        ])))
        .mount(&server)
        .await;
    let p = client.list_positions().await.expect("positions");
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].symbol, "AAPL");
    assert_eq!(p[0].unrealized_pl, Some(dec("10.00")));
}

#[tokio::test]
async fn get_account_decodes_decimals() {
    let (server, client) = server_with_creds().await;
    Mock::given(method("GET"))
        .and(path("/v2/account"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "acct-1",
            "status": "ACTIVE",
            "cash": "10000.00",
            "equity": "12000.00",
            "buying_power": "24000.00",
            "portfolio_value": "12000.00",
            "daytrade_count": 0,
            "pattern_day_trader": false
        })))
        .mount(&server)
        .await;
    let a = client.get_account().await.expect("account");
    assert_eq!(a.cash, dec("10000.00"));
    assert_eq!(a.buying_power, dec("24000.00"));
    assert_eq!(a.pattern_day_trader, Some(false));
}
