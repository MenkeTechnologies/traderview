// Liquidity helpers shared by view + vitest.
//
// Backend body shape: { trades: Trade[], adv: { SYMBOL: decimal } }.
// The frontend takes simplified input (symbol qty net_pnl per line +
// symbol adv per line) and synthesizes full Trade records — most fields
// of the canonical Trade struct aren't relevant for the liquidity
// analyzer (it only reads symbol, qty, net_pnl) but the backend
// deserializer requires them, so we fill with deterministic stubs.

const TOKEN_DELIM = /[\s,]+/;

export function parseTradeLines(text) {
    const trades = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { trades, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (symbol qty net_pnl), got ${parts.length}` });
            continue;
        }
        const symbol = parts[0].toUpperCase();
        const qty = Number(parts[1]);
        const pnl = Number(parts[2]);
        if (!/^[A-Z0-9._-]+$/.test(symbol)) {
            errors.push({ line_no: i + 1, raw, message: `bad symbol "${parts[0]}"` });
            continue;
        }
        if (!Number.isFinite(qty) || qty <= 0) {
            errors.push({ line_no: i + 1, raw, message: `qty must be > 0` });
            continue;
        }
        if (!Number.isFinite(pnl)) {
            errors.push({ line_no: i + 1, raw, message: `net_pnl must be finite` });
            continue;
        }
        trades.push({ symbol, qty, net_pnl: pnl });
    }
    return { trades, errors };
}

export function parseAdvLines(text) {
    const adv = {};
    const errors = [];
    if (typeof text !== 'string') {
        return { adv, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (symbol adv), got ${parts.length}` });
            continue;
        }
        const symbol = parts[0].toUpperCase();
        const advVal = Number(parts[1]);
        if (!/^[A-Z0-9._-]+$/.test(symbol)) {
            errors.push({ line_no: i + 1, raw, message: `bad symbol "${parts[0]}"` });
            continue;
        }
        if (!Number.isFinite(advVal) || advVal <= 0) {
            errors.push({ line_no: i + 1, raw, message: `adv must be > 0` });
            continue;
        }
        adv[symbol] = advVal;
    }
    return { adv, errors };
}

export function validateInputs(trades, adv) {
    if (!Array.isArray(trades) || trades.length === 0) return 'need at least 1 trade';
    if (!adv || Object.keys(adv).length === 0) return 'need at least 1 symbol → ADV mapping';
    const tradeSyms = new Set(trades.map(t => t.symbol));
    const advSyms = new Set(Object.keys(adv));
    const missing = [...tradeSyms].filter(s => !advSyms.has(s));
    if (missing.length === tradeSyms.size) {
        return `no trade symbol has ADV — supplied ADV for [${[...advSyms].join(', ')}], need [${[...tradeSyms].join(', ')}]`;
    }
    return null;
}

// Synthesizes a Trade record matching the backend's required schema.
// Most fields don't affect the liquidity analyzer; placeholders are
// deterministic so the request payload is stable across renders.
export function synthesizeTrade(t, idx) {
    const id = `00000000-0000-4000-8000-${String(idx).padStart(12, '0')}`;
    return {
        id,
        account_id: '00000000-0000-4000-8000-000000000000',
        symbol: t.symbol,
        side: t.net_pnl >= 0 ? 'long' : 'short',
        status: 'closed',
        opened_at: '2024-01-01T00:00:00Z',
        closed_at: '2024-01-01T00:00:00Z',
        qty: t.qty.toString(),
        entry_avg: '100',
        exit_avg: '100',
        gross_pnl: t.net_pnl.toString(),
        fees: '0',
        net_pnl: t.net_pnl.toString(),
        asset_class: 'stock',
        option_type: null,
        strike: null,
        expiration: null,
        multiplier: '1',
        tick_size: null,
        tick_value: null,
        base_ccy: null,
        quote_ccy: null,
        pip_size: null,
        stop_loss: null,
        risk_amount: null,
        initial_target: null,
        mfe: null,
        mae: null,
        rr_planned: null,
        rr_realized: null,
        venue: null,
        order_type: null,
        tif: null,
        notes: null,
        tags: [],
        playbook: null,
        confidence_pre: null,
        confidence_post: null,
        emotion_pre: null,
        emotion_post: null,
        screenshots: [],
        review_status: null,
        executions: [],
    };
}

export function buildBody(trades, adv) {
    const synth = trades.map((t, i) => synthesizeTrade(t, i));
    const advStr = {};
    for (const [k, v] of Object.entries(adv)) {
        advStr[k] = v.toString();   // backend Decimal accepts string
    }
    return { trades: synth, adv: advStr };
}

// Categorizes a single symbol's avg-pct-of-ADV into a liquidity tier
// the trader reads at a glance.
export function liquidityTier(pct) {
    if (!Number.isFinite(pct)) return { label: '—', cls: '' };
    if (pct < 0.001) return { label: 'invisible (<0.1%)',  cls: 'pos' };
    if (pct < 0.01)  return { label: 'normal (0.1-1%)',    cls: 'pos' };
    if (pct < 0.05)  return { label: 'large (1-5%)',       cls: '' };
    if (pct < 0.20)  return { label: 'illiquid (5-20%)',   cls: 'neg' };
    return                  { label: 'whale (>20%)',       cls: 'neg' };
}

// Deterministic demo: 4 symbols of varying ADV → trades that span the
// full liquidity spectrum. AAPL (huge ADV) trades are invisible; ILQD
// (toy small-cap) is whale territory.
export function makeDemoData() {
    const trades = [];
    let n = 0;
    // AAPL: huge ADV, small trades — all <0.1%
    for (let i = 0; i < 20; i++) {
        trades.push({ symbol: 'AAPL', qty: 100 + (i % 5) * 50, net_pnl: (i % 3 - 1) * 50 });
        n++;
    }
    // MSFT: medium ADV, mid trades — 0.1-1%
    for (let i = 0; i < 15; i++) {
        trades.push({ symbol: 'MSFT', qty: 1000 + (i % 4) * 200, net_pnl: (i % 4 - 2) * 75 });
        n++;
    }
    // SMID: small cap, frequent — 1-5%
    for (let i = 0; i < 12; i++) {
        trades.push({ symbol: 'SMID', qty: 5000 + (i % 3) * 1000, net_pnl: (i % 5 - 2) * 100 });
        n++;
    }
    // ILQD: micro cap, whale trades — >5%
    for (let i = 0; i < 6; i++) {
        trades.push({ symbol: 'ILQD', qty: 8000 + (i % 2) * 4000, net_pnl: (i % 3 - 1) * -150 });
        n++;
    }
    void n;
    const adv = {
        AAPL: 50_000_000,
        MSFT: 1_500_000,
        SMID: 250_000,
        ILQD: 80_000,
    };
    return { trades, adv };
}

export function fmtN(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toLocaleString('en-US');
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(3) + '%';
}

export function fmtUSD(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '$';
    return sign + Math.abs(v).toFixed(2);
}
