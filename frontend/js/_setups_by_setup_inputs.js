// Setups-by-setup helpers shared by view + vitest.
//
// Backend body: { trades: Trade[], trade_setups: { trade_id: setup_name } }.
// Returns: SetupStats[] sorted by net_pnl DESC.
//
// Trade is a huge struct but stats_by_setup only uses {id, status, net_pnl,
// gross_pnl, fees, risk_amount}. We build a synthetic Trade per row from a
// simple "<setup> <net_pnl> [risk_amount]" blob — all other fields are
// filler that satisfies the deserialize contract.
//
// Decimals go on the wire as strings per rust_decimal contract.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Per line: "<setup> <net_pnl> [risk_amount]". setup="-" means untagged
// (validates as a trade but won't appear in any stats bucket — used to
// demo the "untagged trades are skipped" behavior).
export function parseSetupTradeBlob(text) {
    const rows = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { rows, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const noComment = stripComment(raw);
        const s = noComment.trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length < 2 || parts.length > 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 or 3 tokens (setup net_pnl [risk]), got ${parts.length}` });
            continue;
        }
        const setup = parts[0];
        const netPnl = Number(parts[1]);
        if (!Number.isFinite(netPnl)) {
            errors.push({ line_no: i + 1, raw, message: t('view.setups_by_setup.parse.net_pnl_finite') });
            continue;
        }
        let risk = null;
        if (parts.length === 3) {
            const r = Number(parts[2]);
            if (!Number.isFinite(r) || r <= 0) {
                errors.push({ line_no: i + 1, raw, message: t('view.setups_by_setup.parse.risk_amount_positive') });
                continue;
            }
            risk = r;
        }
        rows.push({ setup, net_pnl: netPnl, risk_amount: risk });
    }
    return { rows, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

// Build the backend request body from the simplified rows: synthesize a
// minimal-but-valid Trade per row + a UUID-keyed setup map (untagged
// rows omitted from the map).
export function buildBody(rows) {
    const trades = [];
    const setups = {};
    let idCounter = 1;
    const nilAccount = '00000000-0000-0000-0000-000000000000';
    for (const r of rows) {
        const id = makeDeterministicUuid(idCounter++);
        trades.push(syntheticTrade(id, r.net_pnl, r.risk_amount));
        if (r.setup !== '-' && r.setup !== '') {
            setups[id] = r.setup;
        }
        void nilAccount; // (clarity: every trade reuses the nil account)
    }
    return { trades, trade_setups: setups };
}

function syntheticTrade(id, netPnl, riskAmount) {
    return {
        id,
        account_id: '00000000-0000-0000-0000-000000000000',
        symbol: 'X',
        side: 'long',
        status: 'closed',
        opened_at: '2026-01-01T09:30:00.000Z',
        closed_at: '2026-01-01T15:30:00.000Z',
        qty:        '1',
        entry_avg:  '100',
        exit_avg:   '101',
        gross_pnl:  String(netPnl),
        fees:       '0',
        net_pnl:    String(netPnl),
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
        risk_amount: riskAmount != null ? String(riskAmount) : null,
        initial_target: null,
        mfe: null,
        mae: null,
        best_exit_pnl: null,
        exit_efficiency: null,
    };
}

// Deterministic v4-shaped UUID: 8-4-4-4-12 hex. Doesn't need crypto
// randomness — the backend only uses it as an opaque map key.
export function makeDeterministicUuid(n) {
    const hex = String(n).padStart(32, '0');
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;
}

export function validateInputs(rows) {
    if (!Array.isArray(rows) || rows.length === 0)
        return t('view.setups_by_setup.validate.rows_min');
    return null;
}

// Pure-JS mirror of crates/traderview-core/src/setup_catalog.rs::stats_by_setup.
// Only operates on the simplified rows since we control the synthetic
// Trade shape end-to-end. Returns the same stats fields + sort order.
export function localAnalyze(rows) {
    // Bucket by setup name (skip untagged "-" rows).
    const buckets = new Map();
    for (const r of rows) {
        if (r.setup === '-' || r.setup === '') continue;
        if (!buckets.has(r.setup)) buckets.set(r.setup, []);
        buckets.get(r.setup).push(r);
    }
    const out = [];
    for (const [setup, members] of buckets.entries()) {
        out.push(computeOne(setup, members));
    }
    out.sort((a, b) => b.net_pnl - a.net_pnl);
    return out;
}

function computeOne(setup, members) {
    let netPnl = 0, grossPnl = 0, fees = 0;
    let wins = 0, losses = 0, scratches = 0;
    let winSum = 0, lossSum = 0;
    let largestWin = 0, largestLoss = 0;
    let rSum = 0, rCount = 0;
    for (const t of members) {
        const net = t.net_pnl;
        const gross = t.net_pnl;  // we synthesize gross = net (fees=0)
        netPnl += net; grossPnl += gross;
        if (net > 0) {
            wins++; winSum += net;
            if (net > largestWin) largestWin = net;
        } else if (net < 0) {
            losses++; lossSum += net;
            if (net < largestLoss) largestLoss = net;
        } else {
            scratches++;
        }
        if (t.risk_amount != null && t.risk_amount > 0) {
            rSum += net / t.risk_amount;
            rCount++;
        }
    }
    const trades = members.length;
    const winRate = trades > 0 ? wins / trades : 0;
    const avgPnl  = trades > 0 ? netPnl / trades : 0;
    const avgWin  = wins > 0 ? winSum / wins : 0;
    const avgLoss = losses > 0 ? lossSum / losses : 0;
    const lossAbs = Math.abs(lossSum);
    const profitFactor = lossAbs === 0
        ? (winSum === 0 ? 0 : Infinity)
        : winSum / lossAbs;
    return {
        setup, trades, wins, losses, scratches,
        net_pnl: netPnl, gross_pnl: grossPnl, fees,
        win_rate: winRate, avg_pnl: avgPnl, avg_win: avgWin, avg_loss: avgLoss,
        profit_factor: profitFactor,
        expectancy: avgPnl,
        avg_r: rCount > 0 ? rSum / rCount : 0,
        largest_win: largestWin, largest_loss: largestLoss,
    };
}

// Decimal-string-or-number-friendly extractor for the backend response.
// Backend serializes Decimal as strings on the wire.
export function dec(v) {
    if (v == null) return 0;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
}

// Light traffic-light badge by expectancy: above 0 = pos, 0 = neutral.
export function setupBadge(stats) {
    if (!stats) return { label: '—', cls: '' };
    if (stats.avg_pnl > 0) return { label: t('view.setups_by_setup.badge.positive'), cls: 'pos' };
    if (stats.avg_pnl < 0) return { label: t('view.setups_by_setup.badge.negative'), cls: 'neg' };
    return { label: t('view.setups_by_setup.badge.scratch'), cls: '' };
}

// Demo presets that exercise each branch of stats_by_setup.
export function makeDemoRows(kind = 'mixed') {
    switch (kind) {
        case 'single-winner':
            return [
                { setup: 'orb',          net_pnl:  500, risk_amount: 100 },
                { setup: 'orb',          net_pnl:  300, risk_amount: 100 },
                { setup: 'orb',          net_pnl:  200, risk_amount: 100 },
            ];
        case 'single-loser':
            return [
                { setup: 'fade-vwap',    net_pnl: -120, risk_amount: 100 },
                { setup: 'fade-vwap',    net_pnl: -180, risk_amount: 100 },
                { setup: 'fade-vwap',    net_pnl: -200, risk_amount: 100 },
            ];
        case 'mixed':
            return [
                { setup: 'gap-and-go',   net_pnl:  500, risk_amount: 100 },
                { setup: 'gap-and-go',   net_pnl:  300, risk_amount: 100 },
                { setup: 'gap-and-go',   net_pnl: -100, risk_amount: 100 },
                { setup: 'abcd',         net_pnl:  200, risk_amount: 100 },
                { setup: 'abcd',         net_pnl: -150, risk_amount: 100 },
                { setup: 'abcd',         net_pnl: -200, risk_amount: 100 },
                { setup: 'reversal-vwap', net_pnl: 100, risk_amount: 100 },
                { setup: 'reversal-vwap', net_pnl:   0, risk_amount: 100 },
                { setup: 'reversal-vwap', net_pnl: -50, risk_amount: 100 },
            ];
        case 'with-untagged':
            return [
                { setup: 'orb',          net_pnl:  500, risk_amount: 100 },
                { setup: '-',            net_pnl:  999, risk_amount: 100 }, // untagged, excluded
                { setup: 'fade',         net_pnl: -200, risk_amount: 100 },
                { setup: '-',            net_pnl: -111, risk_amount: 100 }, // untagged, excluded
            ];
        case 'all-scratches':
            return [
                { setup: 'breakeven',    net_pnl:    0, risk_amount: 100 },
                { setup: 'breakeven',    net_pnl:    0, risk_amount: 100 },
                { setup: 'breakeven',    net_pnl:    0, risk_amount: 100 },
            ];
        default:
            return makeDemoRows('mixed');
    }
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

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    if (!Number.isFinite(v) || v === Infinity) return '∞';
    return v.toFixed(d);
}

export function fmtPF(v) {
    if (!Number.isFinite(v)) {
        if (v === Infinity) return '∞';
        return '—';
    }
    return v.toFixed(2);
}

export function fmtR(v) {
    if (!Number.isFinite(v)) return '—';
    const s = v >= 0 ? '+' : '';
    return s + v.toFixed(2) + 'R';
}
