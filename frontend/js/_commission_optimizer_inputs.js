// Commission-optimizer helpers shared by view + vitest.
//
// Backend body: { executions: Execution[], tiers: Tier[] }
//   Execution = { qty, notional, actual_fee } — Decimals as strings on wire.
//   Tier      = { name, per_trade_flat, per_share, per_dollar,
//                 min_per_trade, max_per_trade } — same string-decimal contract.
// Returns: OptimizerReport with totals + sorted tiers[] + best_alternative + projected_annual_savings.
//
// Local mirror uses Number throughout for the chart math, then we ship
// Decimals as strings via buildBody.

const TOKEN_DELIM = /[\s,]+/;

// Per line: "<qty> <notional> <actual_fee>". All > 0.
export function parseExecutionBlob(text) {
    const executions = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { executions, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const noComment = stripComment(raw);
        const s = noComment.trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (qty notional actual_fee), got ${parts.length}` });
            continue;
        }
        const qty = Number(parts[0]);
        const notional = Number(parts[1]);
        const fee = Number(parts[2]);
        if (![qty, notional, fee].every(Number.isFinite)) {
            errors.push({ line_no: i + 1, raw, message: 'tokens must be finite numbers' });
            continue;
        }
        if (qty <= 0) {
            errors.push({ line_no: i + 1, raw, message: 'qty must be > 0' });
            continue;
        }
        if (notional <= 0) {
            errors.push({ line_no: i + 1, raw, message: 'notional must be > 0' });
            continue;
        }
        if (fee < 0) {
            errors.push({ line_no: i + 1, raw, message: 'actual_fee must be ≥ 0' });
            continue;
        }
        executions.push({ qty, notional, actual_fee: fee });
    }
    return { executions, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function validateInputs(executions, tiers) {
    if (!Array.isArray(executions) || executions.length === 0)
        return 'need ≥ 1 execution';
    if (!Array.isArray(tiers) || tiers.length === 0)
        return 'need ≥ 1 tier to compare';
    for (let i = 0; i < tiers.length; i++) {
        const t = tiers[i];
        if (!t || typeof t.name !== 'string' || !t.name.trim())
            return `tier[${i}].name required`;
        for (const k of ['per_trade_flat', 'per_share', 'per_dollar', 'min_per_trade', 'max_per_trade']) {
            if (!Number.isFinite(t[k]) || t[k] < 0)
                return `tier[${i}].${k} must be ≥ 0 finite number`;
        }
    }
    return null;
}

// Stringify Decimal fields per rust_decimal contract.
export function buildBody(executions, tiers) {
    return {
        executions: executions.map(e => ({
            qty: String(e.qty),
            notional: String(e.notional),
            actual_fee: String(e.actual_fee),
        })),
        tiers: tiers.map(t => ({
            name: t.name,
            per_trade_flat: String(t.per_trade_flat),
            per_share:      String(t.per_share),
            per_dollar:     String(t.per_dollar),
            min_per_trade:  String(t.min_per_trade),
            max_per_trade:  String(t.max_per_trade),
        })),
    };
}

// Coerce a Decimal-string-or-number field back to a Number for chart math.
export function dec(v) {
    if (v == null) return 0;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
}

// Mirror of crates/traderview-core/src/commission_optimizer.rs::Tier::fee_for.
// Returns the fee a single tier would charge for this execution.
export function feeForTier(tier, exec) {
    const raw = tier.per_trade_flat
              + tier.per_share  * exec.qty
              + tier.per_dollar * exec.notional;
    if (tier.min_per_trade > 0 && raw < tier.min_per_trade) return tier.min_per_trade;
    if (tier.max_per_trade > 0 && raw > tier.max_per_trade) return tier.max_per_trade;
    return raw;
}

// Mirror of commission_optimizer::evaluate. Sorts cheapest first, picks
// best_alternative only when STRICTLY cheaper than actual.
export function localEvaluate(execs, tiers) {
    if (!Array.isArray(execs) || execs.length === 0) {
        return {
            trade_count: 0, total_shares: 0, total_notional: 0,
            actual_total_fee: 0, tiers: [],
            best_alternative: null, projected_annual_savings: 0,
        };
    }
    const tradeCount   = execs.length;
    const totalShares  = execs.reduce((a, e) => a + e.qty, 0);
    const totalNotional = execs.reduce((a, e) => a + e.notional, 0);
    const actualTotal  = execs.reduce((a, e) => a + e.actual_fee, 0);
    const results = (tiers || []).map(t => {
        const total = execs.reduce((a, e) => a + feeForTier(t, e), 0);
        return {
            tier: t.name,
            total_fee: total,
            fee_per_trade: total / tradeCount,
            fee_per_share: totalShares > 0 ? total / totalShares : 0,
            fee_pct_of_notional: totalNotional > 0 ? (total / totalNotional) * 100 : 0,
            delta_vs_actual: total - actualTotal,
        };
    });
    results.sort((a, b) => a.total_fee - b.total_fee);
    let bestAlt = null, annual = 0;
    if (results.length > 0 && results[0].delta_vs_actual < 0) {
        bestAlt = results[0].tier;
        annual  = -results[0].delta_vs_actual * 12;
    }
    return {
        trade_count: tradeCount,
        total_shares: totalShares,
        total_notional: totalNotional,
        actual_total_fee: actualTotal,
        tiers: results,
        best_alternative: bestAlt,
        projected_annual_savings: annual,
    };
}

// Mirror of `default_tiers()` — 3 real-world tiers users compare against.
// Values match the Rust constants byte-for-byte.
export function defaultTiers() {
    return [
        { name: 'IBKR Pro tiered',         per_trade_flat: 0, per_share: 0.0035, per_dollar: 0, min_per_trade: 0.35, max_per_trade: 0 },
        { name: 'Lightspeed Active',       per_trade_flat: 0, per_share: 0.0045, per_dollar: 0, min_per_trade: 1.00, max_per_trade: 0 },
        { name: 'Webull (zero-commission)', per_trade_flat: 0, per_share: 0,      per_dollar: 0, min_per_trade: 0,    max_per_trade: 0 },
    ];
}

// Demo presets — realistic execution profiles per trader archetype.
export function makeDemoExecutions(kind = 'active-retail') {
    switch (kind) {
        case 'active-retail': {
            // 30 fills/month, 100-500 sh each, $5-50k notional, $1 IBKR-Lite-ish fee.
            const out = [];
            for (let i = 0; i < 30; i++) {
                const qty = 100 + ((i * 37) % 5) * 100;
                const px  = 30 + ((i * 17) % 9) * 5;
                const notional = qty * px;
                out.push({ qty, notional, actual_fee: 1.00 });
            }
            return out;
        }
        case 'scalper-heavy': {
            // 200 small fills/month, 50-100 sh each, $5-15k notional, $1 fee.
            // Per-share would beat flat-fee here.
            const out = [];
            for (let i = 0; i < 200; i++) {
                const qty = 50 + ((i * 13) % 6) * 10;
                const notional = qty * (50 + ((i * 7) % 11) * 10);
                out.push({ qty, notional, actual_fee: 1.00 });
            }
            return out;
        }
        case 'options-light': {
            // 20 fills, 1-5 contracts each (1 ct = 100 shares for our purposes),
            // $5-30k notional, $0.65 per-contract fees.
            const out = [];
            for (let i = 0; i < 20; i++) {
                const contracts = 1 + ((i * 7) % 5);
                const qty = contracts;
                const notional = contracts * 2000;
                const fee = 0.65 * contracts;
                out.push({ qty, notional, actual_fee: fee });
            }
            return out;
        }
        case 'webull-zero': {
            // 50 fills, 100-200 sh, ZERO fees (already on commission-free broker).
            const out = [];
            for (let i = 0; i < 50; i++) {
                const qty = 100 + ((i * 13) % 4) * 25;
                const notional = qty * (40 + ((i * 7) % 11) * 5);
                out.push({ qty, notional, actual_fee: 0 });
            }
            return out;
        }
        case 'big-blocks': {
            // 5 huge block trades, 5000-20000 sh each. Per-share rates win here.
            const out = [];
            for (let i = 0; i < 5; i++) {
                const qty = 5000 + i * 3000;
                const notional = qty * 50;
                out.push({ qty, notional, actual_fee: 10.00 });
            }
            return out;
        }
        default:
            return makeDemoExecutions('active-retail');
    }
}

// Verdict badge.
export function savingsBadge(annualSavings) {
    if (!Number.isFinite(annualSavings)) return { key: 'view.commission_optimizer.badge.unknown', cls: '' };
    if (annualSavings <= 0)              return { key: 'view.commission_optimizer.badge.optimal',     cls: 'pos' };
    if (annualSavings < 100)             return { key: 'view.commission_optimizer.badge.marginal',    cls: '' };
    if (annualSavings < 1000)            return { key: 'view.commission_optimizer.badge.meaningful',  cls: 'pos' };
    return { key: 'view.commission_optimizer.badge.significant',     cls: 'pos' };
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtPct(v, d = 3) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d) + '%';
}

export function fmtN(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return Math.trunc(v).toString();
}
