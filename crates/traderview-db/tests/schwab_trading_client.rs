//! Schwab Trader API — wiremock contract tests.
//!
//! Pinned shapes (drift here = silent broker reject):
//!   * POST /trader/v1/accounts/{accountHash}/orders body MUST be
//!     `{orderType, session, duration, orderStrategyType:"SINGLE",
//!       orderLegCollection:[{instruction, quantity, instrument:{symbol, assetType:"EQUITY"}}]}`.
//!   * place_order returns 201 + Location header — body is empty.
//!     Adapter must pull order id out of `Location: /orders/.../<id>`.
//!   * DELETE /accounts/{h}/orders/{id} = cancel.
//!   * 401 triggers ONE auto-refresh via POST {auth_base}/oauth/token
//!     with grant_type=refresh_token; retried call must succeed.
//!   * Refresh-token rotation: new refresh_token in response replaces
//!     the stored one, and the on_token_refresh callback fires.

use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use traderview_db::schwab_trading::{
    Duration_, Instruction, OrderType, PlaceBracket, PlaceOrder, SchwabError, SchwabTrading,
    Session, Tokens,
};
use wiremock::matchers::{body_json_string, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

async fn server_with_tokens(access: &str, refresh: &str) -> (MockServer, SchwabTrading) {
    let server = MockServer::start().await;
    let tokens = Tokens {
        access_token: access.into(),
        refresh_token: refresh.into(),
    };
    let client = SchwabTrading::with_bases(
        format!("{}/trader/v1", server.uri()),
        format!("{}/v1", server.uri()),
        "client-id-abc",
        "client-secret-xyz",
        tokens,
        "ACCT-HASH-1",
    );
    (server, client)
}

#[tokio::test]
async fn place_market_buy_sends_canonical_json_and_extracts_location_id() {
    let (server, client) = server_with_tokens("acc-1", "ref-1").await;
    Mock::given(method("POST"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1/orders"))
        .and(header("Authorization", "Bearer acc-1"))
        .and(body_json_string(
            serde_json::to_string(&serde_json::json!({
                "orderType": "MARKET",
                "session": "NORMAL",
                "duration": "DAY",
                "orderStrategyType": "SINGLE",
                "orderLegCollection": [{
                    "instruction": "BUY",
                    "quantity": 10.0,
                    "instrument": { "symbol": "AAPL", "assetType": "EQUITY" },
                    "comment": "algo-abc"
                }]
            }))
            .unwrap(),
        ))
        .respond_with(
            ResponseTemplate::new(201).insert_header("Location", "/orders/ACCT-HASH-1/9988776655"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceOrder {
        symbol: "AAPL".into(),
        instruction: Instruction::Buy,
        order_type: OrderType::Market,
        quantity: dec("10"),
        duration: Duration_::Day,
        session: Session::Normal,
        price: None,
        comment: Some("algo-abc".into()),
    };
    let resp = client.place_order(&req).await.expect("place ok");
    assert_eq!(resp.order_id.as_deref(), Some("9988776655"));
    assert_eq!(resp.status.as_deref(), Some("Accepted"));
}

#[tokio::test]
async fn limit_order_includes_price_field() {
    let (server, client) = server_with_tokens("acc-1", "ref-1").await;
    Mock::given(method("POST"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1/orders"))
        .and(body_json_string(
            serde_json::to_string(&serde_json::json!({
                "orderType": "LIMIT",
                "session": "NORMAL",
                "duration": "GOOD_TILL_CANCEL",
                "orderStrategyType": "SINGLE",
                "price": 187.5,
                "orderLegCollection": [{
                    "instruction": "SELL",
                    "quantity": 5.0,
                    "instrument": { "symbol": "AAPL", "assetType": "EQUITY" }
                }]
            }))
            .unwrap(),
        ))
        .respond_with(
            ResponseTemplate::new(201).insert_header("Location", "/orders/ACCT-HASH-1/777"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceOrder {
        symbol: "AAPL".into(),
        instruction: Instruction::Sell,
        order_type: OrderType::Limit,
        quantity: dec("5"),
        duration: Duration_::GoodTillCancel,
        session: Session::Normal,
        price: Some(dec("187.5")),
        comment: None,
    };
    let resp = client.place_order(&req).await.expect("limit ok");
    assert_eq!(resp.order_id.as_deref(), Some("777"));
}

#[tokio::test]
async fn auto_refresh_on_401_then_retry() {
    use wiremock::matchers::body_string_contains;
    let (server, client) = server_with_tokens("expired", "ref-1").await;

    // First /orders call: 401 (token expired).
    Mock::given(method("POST"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1/orders"))
        .and(header("Authorization", "Bearer expired"))
        .respond_with(ResponseTemplate::new(401).set_body_string("token expired"))
        .expect(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Refresh exchange — Schwab returns NEW (access, refresh) pair.
    Mock::given(method("POST"))
        .and(path("/v1/oauth/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains("refresh_token=ref-1"))
        .and(header(
            "Authorization",
            // Basic base64("client-id-abc:client-secret-xyz")
            "Basic Y2xpZW50LWlkLWFiYzpjbGllbnQtc2VjcmV0LXh5eg==",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "fresh-1",
            "refresh_token": "ref-2"
        })))
        .expect(1)
        .mount(&server)
        .await;

    // Retry with the fresh access token succeeds.
    Mock::given(method("POST"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1/orders"))
        .and(header("Authorization", "Bearer fresh-1"))
        .respond_with(
            ResponseTemplate::new(201).insert_header("Location", "/orders/ACCT-HASH-1/55"),
        )
        .expect(1)
        .mount(&server)
        .await;

    // Persistence callback should fire with the rotated pair.
    let captured: Arc<StdMutex<Option<Tokens>>> = Arc::new(StdMutex::new(None));
    let cap2 = captured.clone();
    let client = client.on_token_refresh(Arc::new(move |t| {
        *cap2.lock().unwrap() = Some(t);
    }));

    let req = PlaceOrder {
        symbol: "AAPL".into(),
        instruction: Instruction::Buy,
        order_type: OrderType::Market,
        quantity: dec("1"),
        duration: Duration_::Day,
        session: Session::Normal,
        price: None,
        comment: None,
    };
    let resp = client.place_order(&req).await.expect("retry ok");
    assert_eq!(resp.order_id.as_deref(), Some("55"));

    let got = captured.lock().unwrap().clone().expect("callback fired");
    assert_eq!(got.access_token, "fresh-1");
    assert_eq!(got.refresh_token, "ref-2");
}

#[tokio::test]
async fn refresh_failure_surfaces_auth_failed() {
    let (server, client) = server_with_tokens("expired", "expired-refresh").await;
    Mock::given(method("POST"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1/orders"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;
    // Schwab returns 400 when the refresh token itself is dead.
    Mock::given(method("POST"))
        .and(path("/v1/oauth/token"))
        .respond_with(ResponseTemplate::new(400).set_body_string("invalid_grant"))
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceOrder {
        symbol: "AAPL".into(),
        instruction: Instruction::Buy,
        order_type: OrderType::Market,
        quantity: dec("1"),
        duration: Duration_::Day,
        session: Session::Normal,
        price: None,
        comment: None,
    };
    let err = client.place_order(&req).await.unwrap_err();
    assert!(matches!(err, SchwabError::AuthFailed), "got {err:?}");
}

#[tokio::test]
async fn cancel_order_hits_delete_endpoint() {
    let (server, client) = server_with_tokens("acc-1", "ref-1").await;
    Mock::given(method("DELETE"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1/orders/9988"))
        .and(header("Authorization", "Bearer acc-1"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    client.cancel_order("9988").await.expect("cancel ok");
}

#[tokio::test]
async fn get_account_parses_balances() {
    let (server, client) = server_with_tokens("acc-1", "ref-1").await;
    Mock::given(method("GET"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "securitiesAccount": {
                "accountNumber": "12345",
                "type": "MARGIN",
                "currentBalances": {
                    "liquidationValue": 100000.0,
                    "cashBalance": 25000.0,
                    "buyingPower": 200000.0
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let acc = client.get_account().await.expect("get_account ok");
    let sec = acc.securities_account.expect("sec acct present");
    let bal = sec.current_balances.expect("balances present");
    assert_eq!(bal.liquidation_value, Some(100000.0));
    assert_eq!(bal.buying_power, Some(200000.0));
}

#[tokio::test]
async fn buying_power_403_mapped() {
    let (server, client) = server_with_tokens("acc-1", "ref-1").await;
    Mock::given(method("POST"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1/orders"))
        .respond_with(ResponseTemplate::new(403).set_body_string("insufficient buying power"))
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceOrder {
        symbol: "AAPL".into(),
        instruction: Instruction::Buy,
        order_type: OrderType::Market,
        quantity: dec("1000"),
        duration: Duration_::Day,
        session: Session::Normal,
        price: None,
        comment: None,
    };
    let err = client.place_order(&req).await.unwrap_err();
    assert!(
        matches!(err, SchwabError::InsufficientBuyingPower),
        "got {err:?}"
    );
}

#[tokio::test]
async fn bracket_buy_sends_trigger_with_oco_children() {
    let (server, client) = server_with_tokens("acc-1", "ref-1").await;
    Mock::given(method("POST"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1/orders"))
        .and(body_json_string(
            serde_json::to_string(&serde_json::json!({
                "orderType": "MARKET",
                "session": "NORMAL",
                "duration": "DAY",
                "orderStrategyType": "TRIGGER",
                "orderLegCollection": [{
                    "instruction": "BUY",
                    "quantity": 10.0,
                    "instrument": { "symbol": "AAPL", "assetType": "EQUITY" },
                    "comment": "algo-bracket"
                }],
                "childOrderStrategies": [{
                    "orderStrategyType": "OCO",
                    "childOrderStrategies": [
                        {
                            "orderType": "LIMIT",
                            "session": "NORMAL",
                            "duration": "DAY",
                            "orderStrategyType": "SINGLE",
                            "price": 200.0,
                            "orderLegCollection": [{
                                "instruction": "SELL",
                                "quantity": 10.0,
                                "instrument": { "symbol": "AAPL", "assetType": "EQUITY" }
                            }]
                        },
                        {
                            "orderType": "STOP",
                            "session": "NORMAL",
                            "duration": "DAY",
                            "orderStrategyType": "SINGLE",
                            "stopPrice": 180.0,
                            "orderLegCollection": [{
                                "instruction": "SELL",
                                "quantity": 10.0,
                                "instrument": { "symbol": "AAPL", "assetType": "EQUITY" }
                            }]
                        }
                    ]
                }]
            }))
            .unwrap(),
        ))
        .respond_with(
            ResponseTemplate::new(201).insert_header("Location", "/orders/ACCT-HASH-1/BRK-1"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceBracket {
        symbol: "AAPL".into(),
        instruction: Instruction::Buy,
        order_type: OrderType::Market,
        quantity: dec("10"),
        duration: Duration_::Day,
        session: Session::Normal,
        entry_price: None,
        take_profit_price: dec("200"),
        stop_loss_price: dec("180"),
        comment: Some("algo-bracket".into()),
    };
    let resp = client.place_bracket(&req).await.expect("bracket ok");
    assert_eq!(resp.order_id.as_deref(), Some("BRK-1"));
}

#[tokio::test]
async fn bracket_short_uses_buy_to_cover_exit() {
    let (server, client) = server_with_tokens("acc-1", "ref-1").await;
    // Pin only the exit-leg instruction shape — full envelope match in
    // the BUY test above. Here we just confirm the SELL_SHORT entry
    // produces BUY_TO_COVER exits, not SELL.
    Mock::given(method("POST"))
        .and(path("/trader/v1/accounts/ACCT-HASH-1/orders"))
        .and(wiremock::matchers::body_string_contains(
            "\"instruction\":\"SELL_SHORT\"",
        ))
        .and(wiremock::matchers::body_string_contains(
            "\"instruction\":\"BUY_TO_COVER\"",
        ))
        .respond_with(
            ResponseTemplate::new(201).insert_header("Location", "/orders/ACCT-HASH-1/BRK-2"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let req = PlaceBracket {
        symbol: "TSLA".into(),
        instruction: Instruction::SellShort,
        order_type: OrderType::Market,
        quantity: dec("5"),
        duration: Duration_::Day,
        session: Session::Normal,
        entry_price: None,
        take_profit_price: dec("180"),
        stop_loss_price: dec("220"),
        comment: None,
    };
    let resp = client.place_bracket(&req).await.expect("short bracket ok");
    assert_eq!(resp.order_id.as_deref(), Some("BRK-2"));
}
