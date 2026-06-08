//! IBKR Client Portal Web API — wiremock contract tests.
//!
//! Pinned shapes (drift here = silent broker reject):
//!   * POST /iserver/account/{id}/orders body MUST be
//!     `{"orders":[{conid,side,orderType,quantity,tif,price?,cOID?}]}`.
//!   * place_order returns an ARRAY, even when only one order is sent.
//!   * resolve_stock_conid POSTs to /iserver/secdef/search with
//!     `{symbol, name:false, secType:"STK"}`.
//!   * cancel_order DELETEs /iserver/account/{id}/order/{order_id}.
//!   * 401 maps to AuthFailed; 403 + "buying power" → InsufficientBuyingPower.

use rust_decimal::Decimal;
use std::str::FromStr;
use traderview_db::ibkr_trading::{
    IbkrError, IbkrTrading, OrderSide, OrderType, PlaceOrder, Tif,
};
use wiremock::matchers::{body_json_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

async fn server_with_token() -> (MockServer, IbkrTrading) {
    let server = MockServer::start().await;
    let client = IbkrTrading::with_base(server.uri(), Some("BEARER123".into()), "DU1234567");
    (server, client)
}

#[tokio::test]
async fn place_market_order_wraps_in_orders_array() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .and(header("Authorization", "Bearer BEARER123"))
        .and(header("Accept", "application/json"))
        .and(body_json_string(serde_json::to_string(&serde_json::json!({
            "orders": [{
                "conid": 265598,
                "side": "BUY",
                "orderType": "MKT",
                "quantity": 10.0,
                "tif": "DAY",
                "cOID": "algo-abc",
            }]
        })).unwrap()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "order_id": "1234567", "local_order_id": "loc-1", "order_status": "Submitted" }
        ])))
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceOrder {
        conid: 265598,
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: dec("10"),
        tif: Tif::Day,
        price: None,
        client_order_id: Some("algo-abc".into()),
    };
    let resp = client.place_order(&req).await.expect("place ok");
    assert_eq!(resp.len(), 1);
    assert_eq!(resp[0].order_id.as_deref(), Some("1234567"));
    assert_eq!(resp[0].order_status.as_deref(), Some("Submitted"));
}

#[tokio::test]
async fn place_limit_order_includes_price() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .and(body_json_string(serde_json::to_string(&serde_json::json!({
            "orders": [{
                "conid": 265598,
                "side": "SELL",
                "orderType": "LMT",
                "quantity": 5.0,
                "tif": "GTC",
                "price": 187.5,
            }]
        })).unwrap()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "order_id": "999", "order_status": "PendingSubmit" }
        ])))
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceOrder {
        conid: 265598,
        side: OrderSide::Sell,
        order_type: OrderType::Limit,
        quantity: dec("5"),
        tif: Tif::Gtc,
        price: Some(dec("187.5")),
        client_order_id: None,
    };
    let resps = client.place_order(&req).await.expect("place ok");
    assert_eq!(resps[0].order_status.as_deref(), Some("PendingSubmit"));
}

#[tokio::test]
async fn cancel_order_hits_delete_endpoint() {
    let (server, client) = server_with_token().await;
    Mock::given(method("DELETE"))
        .and(path("/iserver/account/DU1234567/order/1234567"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"msg": "ok"})))
        .expect(1)
        .mount(&server)
        .await;
    client.cancel_order("1234567").await.expect("cancel ok");
}

#[tokio::test]
async fn auth_failed_on_401() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .respond_with(ResponseTemplate::new(401).set_body_string("session not authenticated"))
        .mount(&server)
        .await;
    let req = PlaceOrder {
        conid: 1,
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: dec("1"),
        tif: Tif::Day,
        price: None,
        client_order_id: None,
    };
    let err = client.place_order(&req).await.unwrap_err();
    assert!(matches!(err, IbkrError::AuthFailed));
}

#[tokio::test]
async fn buying_power_403_mapped() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_string("insufficient buying power for order"),
        )
        .mount(&server)
        .await;
    let req = PlaceOrder {
        conid: 1,
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: dec("1000"),
        tif: Tif::Day,
        price: None,
        client_order_id: None,
    };
    let err = client.place_order(&req).await.unwrap_err();
    assert!(matches!(err, IbkrError::InsufficientBuyingPower));
}

#[tokio::test]
async fn resolve_stock_conid_picks_us_listing() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/iserver/secdef/search"))
        .and(body_json_string(serde_json::to_string(&serde_json::json!({
            "symbol": "AAPL",
            "name": false,
            "secType": "STK"
        })).unwrap()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "conid": 11111, "description": "AAPL.XETRA.STK" },
            { "conid": 265598, "description": "AAPL.NASDAQ.STK" },
            { "conid": 22222, "description": "AAPL.LSE.STK" }
        ])))
        .expect(1)
        .mount(&server)
        .await;
    let conid = client.resolve_stock_conid("AAPL").await.expect("resolved");
    assert_eq!(conid, 265598, "should prefer NASDAQ listing");
}

#[tokio::test]
async fn resolve_stock_conid_errors_when_empty() {
    let (server, client) = server_with_token().await;
    Mock::given(method("POST"))
        .and(path("/iserver/secdef/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    let err = client.resolve_stock_conid("ZZZZZ").await.unwrap_err();
    assert!(matches!(err, IbkrError::InvalidRequest(_)));
}

#[tokio::test]
async fn get_summary_parses_amounts() {
    let (server, client) = server_with_token().await;
    Mock::given(method("GET"))
        .and(path("/portfolio/DU1234567/summary"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "netliquidation": { "amount": 100000.0, "currency": "USD", "timestamp": 1717000000 },
            "totalcashvalue": { "amount": 50000.0, "currency": "USD" },
            "buyingpower": { "amount": 200000.0, "currency": "USD" },
            "availablefunds": { "amount": 99500.0, "currency": "USD" }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let s = client.get_summary().await.expect("summary ok");
    assert_eq!(s.net_liquidation.and_then(|v| v.amount), Some(100000.0));
    assert_eq!(s.buying_power.and_then(|v| v.amount), Some(200000.0));
}

#[tokio::test]
async fn cookie_jar_auth_when_no_bearer() {
    // No bearer token configured — we should still hit the endpoint
    // (cookie-jar auth path used with local gateway). Authorization
    // header must NOT be sent.
    let server = MockServer::start().await;
    let client = IbkrTrading::with_base(server.uri(), None, "DU1234567");
    Mock::given(method("GET"))
        .and(path("/portfolio/DU1234567/summary"))
        .and(wiremock::matchers::header_exists("accept"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "netliquidation": { "amount": 5000.0 }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let s = client.get_summary().await.expect("summary ok");
    assert_eq!(s.net_liquidation.and_then(|v| v.amount), Some(5000.0));
}
