//! Convertible bond pricing — binomial tree (Cox-Ross-Rubinstein) on
//! the underlying stock with bond-conversion + bond-redemption boundary
//! conditions.
//!
//! At each node the bond value is max(
//!   conversion_ratio · stock,      // convert NOW
//!   continuation_value,            // keep holding the bond
//!   put_price?                     // optional bondholder put right
//! ), and capped at call_price if the issuer has a call right.
//!
//! At maturity:
//!   value = max(redemption_value, conversion_ratio · stock)
//!
//! Pure compute. Simplified single-factor model (no stochastic interest
//! rates or credit spread — those would extend to a 2-factor tree).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConvertibleBondInputs {
    pub spot: f64,
    pub conversion_ratio: f64,
    pub redemption_face_value: f64,
    pub time_to_expiry: f64,
    pub risk_free: f64,
    pub dividend_yield: f64,
    pub sigma: f64,
    pub n_steps: usize,
    /// Optional issuer call price; bond is callable at this price after
    /// the call-protection period. None = not callable.
    pub call_price: Option<f64>,
    /// Optional bondholder put price; holder can sell back at this
    /// price (typically par). None = no put right.
    pub put_price: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConvertibleBondReport {
    pub price: f64,
    pub conversion_value_today: f64,
    pub parity: f64,
}

pub fn price(inputs: &ConvertibleBondInputs) -> Option<ConvertibleBondReport> {
    if !inputs.spot.is_finite()
        || inputs.spot <= 0.0
        || !inputs.conversion_ratio.is_finite()
        || inputs.conversion_ratio <= 0.0
        || !inputs.redemption_face_value.is_finite()
        || inputs.redemption_face_value <= 0.0
        || !inputs.time_to_expiry.is_finite()
        || inputs.time_to_expiry <= 0.0
        || !inputs.risk_free.is_finite()
        || !inputs.dividend_yield.is_finite()
        || !inputs.sigma.is_finite()
        || inputs.sigma <= 0.0
        || !(1..=5_000).contains(&inputs.n_steps)
    {
        return None;
    }
    if let Some(c) = inputs.call_price {
        if !c.is_finite() || c <= 0.0 {
            return None;
        }
    }
    if let Some(p) = inputs.put_price {
        if !p.is_finite() || p <= 0.0 {
            return None;
        }
    }
    let dt = inputs.time_to_expiry / inputs.n_steps as f64;
    let u = (inputs.sigma * dt.sqrt()).exp();
    let d = 1.0 / u;
    let disc = (-inputs.risk_free * dt).exp();
    let drift = ((inputs.risk_free - inputs.dividend_yield) * dt).exp();
    let p_up = (drift - d) / (u - d);
    if !(0.0..=1.0).contains(&p_up) {
        return None;
    }
    let q = 1.0 - p_up;
    // Terminal payoffs.
    let mut values = vec![0.0_f64; inputs.n_steps + 1];
    for (j, slot) in values.iter_mut().enumerate() {
        let s = inputs.spot * u.powi((inputs.n_steps as i32) - (j as i32) * 2);
        let convert = inputs.conversion_ratio * s;
        *slot = convert.max(inputs.redemption_face_value);
    }
    for step in (0..inputs.n_steps).rev() {
        for j in 0..=step {
            let s = inputs.spot * u.powi((step as i32) - (j as i32) * 2);
            let continuation = disc * (p_up * values[j] + q * values[j + 1]);
            let convert = inputs.conversion_ratio * s;
            let mut v = continuation.max(convert);
            if let Some(p_put) = inputs.put_price {
                v = v.max(p_put);
            }
            if let Some(p_call) = inputs.call_price {
                v = v.min(p_call.max(convert)); // issuer call capped at max(call, convert)
            }
            values[j] = v;
        }
    }
    Some(ConvertibleBondReport {
        price: values[0],
        conversion_value_today: inputs.conversion_ratio * inputs.spot,
        parity: inputs.conversion_ratio * inputs.spot / inputs.redemption_face_value * 100.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_inputs() -> ConvertibleBondInputs {
        ConvertibleBondInputs {
            spot: 50.0,
            conversion_ratio: 2.0,
            redemption_face_value: 100.0,
            time_to_expiry: 1.0,
            risk_free: 0.05,
            dividend_yield: 0.0,
            sigma: 0.30,
            n_steps: 200,
            call_price: None,
            put_price: None,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            let mut i = default_inputs();
            i.spot = bad;
            assert!(price(&i).is_none());
            let mut i = default_inputs();
            i.conversion_ratio = bad;
            assert!(price(&i).is_none());
            let mut i = default_inputs();
            i.sigma = bad;
            assert!(price(&i).is_none());
            let mut i = default_inputs();
            i.redemption_face_value = bad;
            assert!(price(&i).is_none());
        }
        let mut i = default_inputs();
        i.n_steps = 0;
        assert!(price(&i).is_none());
    }

    #[test]
    fn deep_itm_bond_dominated_by_conversion_value() {
        // Spot = 100, conv ratio = 2 → conversion value = 200 > face 100.
        // Bond should price near max(conversion, continuation) ≈ 200+.
        let mut i = default_inputs();
        i.spot = 100.0;
        let r = price(&i).unwrap();
        assert!(r.price >= 195.0);
        assert!(r.conversion_value_today == 200.0);
    }

    #[test]
    fn deep_otm_bond_priced_near_face_value() {
        // Spot = 10 → conversion = 20 << face 100. Bond floor at face.
        let mut i = default_inputs();
        i.spot = 10.0;
        let r = price(&i).unwrap();
        assert!(
            r.price >= 90.0, // some PV-discounting expected
            "OTM bond should price near face, got {}",
            r.price
        );
    }

    #[test]
    fn put_floor_raises_price() {
        let plain = price(&default_inputs()).unwrap();
        let mut with_put = default_inputs();
        with_put.put_price = Some(95.0);
        let put = price(&with_put).unwrap();
        assert!(put.price >= plain.price - 1e-6);
    }

    #[test]
    fn issuer_call_caps_upside() {
        // Set call price slightly above conversion value for ITM scenarios.
        let mut callable = default_inputs();
        callable.spot = 80.0; // conversion = 160
        callable.call_price = Some(150.0);
        let r_callable = price(&callable).unwrap();
        let mut uncallable = callable;
        uncallable.call_price = None;
        let r_uncallable = price(&uncallable).unwrap();
        assert!(r_callable.price <= r_uncallable.price + 1e-6);
    }

    #[test]
    fn higher_vol_inflates_atm_convertible() {
        let mut low = default_inputs();
        low.spot = 50.0;
        low.sigma = 0.15;
        let mut high = default_inputs();
        high.spot = 50.0;
        high.sigma = 0.50;
        assert!(price(&high).unwrap().price > price(&low).unwrap().price);
    }

    #[test]
    fn longer_expiry_inflates_atm_convertible() {
        let mut short = default_inputs();
        short.time_to_expiry = 0.25;
        let mut long = default_inputs();
        long.time_to_expiry = 5.0;
        assert!(price(&long).unwrap().price > price(&short).unwrap().price);
    }

    #[test]
    fn parity_in_percent_terms() {
        let i = default_inputs();
        let r = price(&i).unwrap();
        // Parity = conversion_value / face × 100. Spot=50, conv_ratio=2,
        // conv_value = 100. Face = 100 → parity = 100.
        assert!((r.parity - 100.0).abs() < 1e-9);
    }
}
