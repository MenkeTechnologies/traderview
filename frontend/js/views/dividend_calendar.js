// Dividend Yield Calendar — for a user-supplied symbol list, fetch
// each symbol's dividend data in parallel via /symbols/:sym/dividends,
// extract upcoming ex-date/amount/yield, and render a sortable
// calendar table filtered by horizon (next 7 / 14 / 30 / 60 / 180
// days).
//
// Data source: the existing per-symbol dividends endpoint (Yahoo
// quoteSummary blob). No additional backend wiring required.
//
// Symbol input: textarea (one per line/comma/space) OR pre-populate
// from the user's default watchlist via the "load from watchlist"
// button if their account has one.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import {
    parseSymbolList, extractDividend, sortByExDate,
    filterByHorizon, daysBetween,
    fmtDate, fmtYield, fmtAmount,
} from '../_dividend_calendar_inputs.js';

const DEFAULT_SYMBOLS = `# One symbol per token (line / comma / space separated).
# Demo: a handful of well-known dividend payers.
KO  PG  JNJ  XOM  CVX  T  VZ  MCD  WMT  PEP  HD  IBM  PFE  MRK  ABBV
`;

const HORIZON_OPTIONS = [
    { value: 7,    n: 7 },
    { value: 14,   n: 14 },
    { value: 30,   n: 30 },
    { value: 60,   n: 60 },
    { value: 180,  key: 'view.dividend_calendar.horizon.next_6_months' },
    { value: 'all', key: 'view.dividend_calendar.horizon.all' },
];
function horizonLabel(o) {
    return o.key ? t(o.key) : t('view.dividend_calendar.horizon.next_n_days', { n: o.n });
}

let state = {
    text: DEFAULT_SYMBOLS,
    horizon: 30,
    lastRows: null,
};

export async function renderDividendCalendar(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.dividend_calendar.h1.dividend_yield_calendar" class="view-title">// DIVIDEND YIELD CALENDAR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.dividend_calendar.h2.symbols">Symbols</h2>
            <textarea id="dc-text" rows="5"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
            <div class="inline-form" style="margin-top:10px">
                <label><span data-i18n="view.dividend_calendar.label.horizon">Horizon</span>
                    <select id="dc-horizon">
                        ${HORIZON_OPTIONS.map(o =>
                            `<option value="${o.value}" ${o.value === state.horizon ? 'selected' : ''}>${esc(horizonLabel(o))}</option>`
                        ).join('')}
                    </select></label>
                <button data-i18n="view.dividend_calendar.btn.load_from_watchlist" id="dc-load-watchlist" class="secondary" type="button">Load from watchlist</button>
                <button data-i18n="view.dividend_calendar.btn.fetch_dividends" id="dc-run" class="primary" type="button">Fetch dividends</button>
            </div>
            <p data-i18n="view.dividend_calendar.hint.pulls_per_symbol_dividend_data_in_parallel_from_th" class="muted">
                Pulls per-symbol dividend data in parallel from the research backend.
                Symbols without dividend data (non-payers, ETFs, ADRs) are silently
                skipped. Past-dated ex-dates are filtered out.
            </p>
        </div>

        <div id="dc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.dividend_calendar.h2.upcoming_dividends">Upcoming dividends</h2>
            <div id="dc-table"></div>
        </div>

        <div id="dc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    wireForm(mount, tok);
    void fmt;
}

function wireForm(mount, tok) {
    document.getElementById('dc-run').addEventListener('click', () => {
        state.text = document.getElementById('dc-text').value;
        const h = document.getElementById('dc-horizon').value;
        state.horizon = h === 'all' ? 'all' : Number(h);
        void runFetch(mount, tok);
    });
    document.getElementById('dc-load-watchlist').addEventListener('click', () => {
        void loadFromWatchlist(mount, tok);
    });
    document.getElementById('dc-horizon').addEventListener('change', e => {
        const h = e.target.value;
        state.horizon = h === 'all' ? 'all' : Number(h);
        // Re-render from cached rows without a refetch.
        if (state.lastRows) renderTable(state.lastRows);
    });
}

async function loadFromWatchlist(mount, tok) {
    hideErr();
    try {
        const wls = await api.watchlists();
        if (!viewIsCurrent(tok)) return;
        if (!Array.isArray(wls) || wls.length === 0) {
            showErr(t('view.dividend_calendar.err.no_watchlists_found_add_one_under_the_watchlists_v'));
            return;
        }
        const first = wls[0];
        const syms = await api.watchlistSymbols(first.id);
        if (!viewIsCurrent(tok)) return;
        const tokens = (syms || []).map(s => s.symbol || s).filter(Boolean);
        const text = `# Loaded from watchlist "${first.name}"\n${tokens.join('  ')}\n`;
        document.getElementById('dc-text').value = text;
        state.text = text;
    } catch (e) {
        showErr(t('view.dividend_calendar.error.watchlist_load', { msg: e.message || e }));
    }
}

async function runFetch(mount, tok) {
    hideErr();
    const symbols = parseSymbolList(state.text);
    if (symbols.length === 0) {
        showErr(t('view.dividend_calendar.err.no_symbols_parsed_from_input'));
        return;
    }
    document.getElementById('dc-table').innerHTML = `<div class="boot">${esc(t('view.dividend_calendar.hint.fetching'))}</div>`;

    // Parallel fetch, but cap concurrency to avoid hammering the backend.
    const rows = await fetchWithConcurrencyLimit(symbols, 8, async (sym) => {
        try {
            const payload = await api.symbolDividends(sym);
            return extractDividend(sym, payload);
        } catch (_e) {
            return null;
        }
    });
    if (!viewIsCurrent(tok)) return;

    const valid = rows.filter(r => r !== null);
    state.lastRows = valid;
    renderSummary(symbols.length, valid);
    renderTable(valid);
}

// Bounded-concurrency mapper. Returns results in original-input order.
async function fetchWithConcurrencyLimit(items, limit, worker) {
    const out = new Array(items.length);
    let next = 0;
    const runners = Array.from({ length: Math.min(limit, items.length) }, async () => {
        while (true) {
            const i = next++;
            if (i >= items.length) return;
            out[i] = await worker(items[i]);
        }
    });
    await Promise.all(runners);
    return out;
}

function renderSummary(requested, rows) {
    const now = new Date();
    const filtered = state.horizon === 'all'
        ? sortByExDate(rows).filter(r => r.ex_date && r.ex_date >= now)
        : filterByHorizon(sortByExDate(rows), now, state.horizon);
    const yields = rows.map(r => r.yield).filter(y => Number.isFinite(y));
    const avgYield = yields.length
        ? yields.reduce((a, b) => a + b, 0) / yields.length
        : NaN;
    const maxYield = yields.length ? Math.max(...yields) : NaN;
    document.getElementById('dc-summary').innerHTML = [
        card(t('view.dividend_calendar.card.symbols_requested'), String(requested)),
        card(t('view.dividend_calendar.card.dividend_payers_found'), String(rows.length)),
        card(t('view.dividend_calendar.card.in_horizon', { window: state.horizon === 'all' ? t('view.dividend_calendar.horizon.all_upcoming') : state.horizon + 'd' }), String(filtered.length)),
        card(t('view.dividend_calendar.card.avg_yield_paying_set'), fmtYield(avgYield)),
        card(t('view.dividend_calendar.card.max_yield'), fmtYield(maxYield)),
    ].join('');
}

function card(label, value) {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value">${esc(value)}</div>
    </div>`;
}

function renderTable(rows) {
    const now = new Date();
    const sorted = sortByExDate(rows);
    const visible = state.horizon === 'all'
        ? sorted.filter(r => r.ex_date && r.ex_date >= now)
        : filterByHorizon(sorted, now, state.horizon);

    if (visible.length === 0) {
        document.getElementById('dc-table').innerHTML =
            `<div class="boot">${esc(t('view.dividend_calendar.empty.no_upcoming'))}</div>`;
        return;
    }

    const rowHtml = visible.map(r => {
        const days = r.ex_date ? daysBetween(now, r.ex_date) : null;
        return `<tr>
            <td>${esc(r.symbol)}</td>
            <td>${esc(fmtDate(r.ex_date))}</td>
            <td class="dc-days">${days == null ? '—' : days + 'd'}</td>
            <td>${esc(fmtDate(r.pay_date))}</td>
            <td class="dc-amount">${esc(fmtAmount(r.amount))}</td>
            <td class="dc-yield">${esc(fmtYield(r.yield))}</td>
            <td>${esc(fmtAmount(r.last_div_amount))}</td>
            <td>${esc(fmtDate(r.last_div_date))}</td>
        </tr>`;
    }).join('');

    document.getElementById('dc-table').innerHTML = `
        <table class="trades dc-table">
            <thead><tr>
                <th data-i18n="view.dividend_calendar.th.symbol">Symbol</th>
                <th data-i18n="view.dividend_calendar.th.ex_date">Ex-date</th>
                <th data-i18n="view.dividend_calendar.th.days_to_ex">Days to ex</th>
                <th data-i18n="view.dividend_calendar.th.pay_date">Pay date</th>
                <th data-i18n="view.dividend_calendar.th.amount_yr">Amount / yr</th>
                <th data-i18n="view.dividend_calendar.th.indicated_yield">Indicated yield</th>
                <th data-i18n="view.dividend_calendar.th.last_paid_amount">Last paid (amount)</th>
                <th data-i18n="view.dividend_calendar.th.last_paid_date">Last paid (date)</th>
            </tr></thead>
            <tbody>${rowHtml}</tbody>
        </table>`;
}

function showErr(msg) {
    const el = document.getElementById('dc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('dc-err').style.display = 'none'; }
