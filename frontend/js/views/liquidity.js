// Liquidity view — per-symbol position-vs-ADV analyzer + pnl-by-ADV-bucket.
//
// Answers two questions:
//   1. Per symbol — what % of ADV does my average trade consume?
//   2. By position-size bucket — where am I making/losing money?
//      (Most traders' P&L cliff sits at the same bucket where slippage cliffs.)

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTradeLines, parseAdvLines, validateInputs, buildBody,
    liquidityTier, makeDemoData, fmtN, fmtPct, fmtUSD,
} from '../_liquidity_inputs.js';

import { t } from '../i18n.js';
let state = { tradesText: '', advText: '' };

export async function renderLiquidity(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.liquidity.h1.liquidity_position_vs_adv" class="view-title">// LIQUIDITY · POSITION vs ADV</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.liquidity.h2.trades">Trades</h2>
            <p class="muted" data-i18n="view.liquidity.hint.format">One line per trade: symbol qty net_pnl. Negative pnl = loss. Other Trade fields are auto-filled.</p>
            <textarea id="lq-trades" rows="8" placeholder="AAPL 100 75&#10;MSFT 2000 -150&#10;..."></textarea>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.liquidity.h2.adv_avg_daily_volume_per_symbol">ADV (avg daily volume per symbol)</h2>
            <textarea id="lq-adv" rows="4" placeholder="AAPL 50000000&#10;MSFT 1500000&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.liquidity.btn.load_demo_4_symbols_53_trades" id="lq-demo" class="secondary" type="button">Load demo (4 symbols, 53 trades)</button>
                <button data-i18n="view.liquidity.btn.clear" id="lq-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.liquidity.btn.analyze" id="lq-run" class="primary" type="button">Analyze</button>
            </div>
            <p data-i18n="view.liquidity.hint.symbols_not_in_the_adv_table_are_silently_dropped_" class="muted">Symbols not in the ADV table are silently dropped from
                bucket analysis. Per-symbol rows still show their qty / pnl regardless.</p>
        </div>

        <div id="lq-errors" class="boot" style="display:none"></div>
        <div id="lq-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.liquidity.h2.per_symbol_breakdown">Per-symbol breakdown</h2>
            <div id="lq-rows"></div>
            <p data-i18n="view.liquidity.hint.each_row_symbol_trade_count_total_qty_avg_qty_trad" class="muted">Each row: symbol · trade count · total qty · avg qty/trade ·
                ADV · avg % of ADV · liquidity tier · net pnl. Tiers cut at 0.1% / 1% / 5% / 20%.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.liquidity.h2.pandl_and_win_rate_by_adv_bucket">P&amp;L &amp; win rate by ADV bucket</h2>
            <div id="lq-buckets"></div>
            <p data-i18n="view.liquidity.hint.if_your_wins_concentrate_in_small_pct_buckets_and_" class="muted">If your wins concentrate in small-pct buckets and losses
                concentrate in large-pct buckets, you have a sizing problem — not an edge problem.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.liquidity.h2.adv_pnl_chart">Avg %ADV vs net P&amp;L (per symbol)</h2>
            <div id="lq-chart" style="width:100%;height:240px"></div>
        </div>

        <div id="lq-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('lq-demo').addEventListener('click', () => {
        const { trades, adv } = makeDemoData();
        document.getElementById('lq-trades').value =
            trades.map(t => `${t.symbol} ${t.qty} ${t.net_pnl}`).join('\n');
        document.getElementById('lq-adv').value =
            Object.entries(adv).map(([s, v]) => `${s} ${v}`).join('\n');
    });
    document.getElementById('lq-clear').addEventListener('click', () => {
        document.getElementById('lq-trades').value = '';
        document.getElementById('lq-adv').value = '';
    });
    document.getElementById('lq-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.tradesText = document.getElementById('lq-trades').value;
    state.advText = document.getElementById('lq-adv').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('lq-errors');
    errs.style.display = 'none';

    const { trades, errors: tradeErrs } = parseTradeLines(state.tradesText);
    const { adv,    errors: advErrs }   = parseAdvLines(state.advText);
    const allErrs = [...tradeErrs.map(e => ({ ...e, src: 'trades' })),
                     ...advErrs.map(e => ({ ...e, src: 'adv' }))];
    if (allErrs.length) {
        const head = allErrs.slice(0, 8).map(e =>
            t('common.parse_error_inline_src', { src: e.src, line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = allErrs.length > 8 ? `<br>${esc(t('common.and_n_more', { n: allErrs.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: allErrs.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
    }
    const err = validateInputs(trades, adv);
    if (err) { showErr(err); return; }

    let res;
    try {
        res = await api.microLiquidity(buildBody(trades, adv));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res, trades);
    renderRows(res);
    renderBuckets(res);
    renderAdvPnlChart(res);
}

function renderAdvPnlChart(report) {
    const el = document.getElementById('lq-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (report && report.rows ? report.rows : [])
        .filter(r => Number.isFinite(Number(r.avg_pct_of_adv)) && Number.isFinite(Number(r.net_pnl)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.liquidity.empty_chart">${esc(t('view.liquidity.empty_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(a.avg_pct_of_adv) - Number(b.avg_pct_of_adv));
    const xs = rows.map(r => Number(r.avg_pct_of_adv) * 100);
    const ys = rows.map(r => Number(r.net_pnl));
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: t('view.liquidity.chart.pct_adv') },
            { label: t('view.liquidity.chart.pnl'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.liquidity.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 } ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderSummary(report, trades) {
    const symCount = report.rows.length;
    const totalQty = report.rows.reduce((a, r) => a + Number(r.total_qty || 0), 0);
    const totalPnl = report.rows.reduce((a, r) => a + Number(r.net_pnl || 0), 0);
    const matchedSym = report.rows.filter(r => r.avg_daily_volume != null).length;
    document.getElementById('lq-summary').innerHTML = [
        card(t('view.liquidity.card.trades'),          fmtN(trades.length)),
        card(t('view.liquidity.card.symbols'),         String(symCount)),
        card(t('view.liquidity.card.matched_adv'),     `${matchedSym} / ${symCount}`),
        card(t('view.liquidity.card.total_qty'),       fmtN(totalQty)),
        card(t('view.liquidity.card.total_net_p_l'),   fmtUSD(totalPnl), totalPnl >= 0 ? 'pos' : 'neg'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderRows(report) {
    const wrap = document.getElementById('lq-rows');
    if (!report.rows.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.liquidity.empty.rows">No rows.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.liquidity.th.symbol">Symbol</th><th data-i18n="view.liquidity.th.trades">Trades</th><th data-i18n="view.liquidity.th.total_qty">Total qty</th><th data-i18n="view.liquidity.th.avg_qty">Avg qty</th>
                <th data-i18n="view.liquidity.th.adv">ADV</th><th data-i18n="view.liquidity.th.avg_adv">Avg %ADV</th><th data-i18n="view.liquidity.th.tier">Tier</th><th data-i18n="view.liquidity.th.net_pandl">Net P&amp;L</th>
            </tr></thead>
            <tbody>
                ${report.rows.map(r => {
                    const pct = r.avg_pct_of_adv;
                    const tier = liquidityTier(pct);
                    const pnl = Number(r.net_pnl || 0);
                    return `<tr>
                        <td>${esc(r.symbol)}</td>
                        <td>${esc(fmtN(r.trades))}</td>
                        <td>${esc(fmtN(Number(r.total_qty)))}</td>
                        <td>${esc(fmtN(Number(r.avg_qty_per_trade)))}</td>
                        <td>${esc(r.avg_daily_volume == null ? '—' : fmtN(Number(r.avg_daily_volume)))}</td>
                        <td>${esc(pct == null ? '—' : fmtPct(pct))}</td>
                        <td class="${tier.cls}">${esc(tier.label)}</td>
                        <td class="${pnl >= 0 ? 'pos' : 'neg'}">${esc(fmtUSD(pnl))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function renderBuckets(report) {
    const wrap = document.getElementById('lq-buckets');
    if (!report.buckets.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.liquidity.empty.buckets">No bucketable trades (no ADV matches).</div>`;
        return;
    }
    const maxAbs = Math.max(...report.buckets.map(b => Math.abs(Number(b.net_pnl || 0))), 1);
    wrap.innerHTML = report.buckets.map(b => {
        const pnl = Number(b.net_pnl || 0);
        if (b.trades === 0) {
            return `
                <div class="is-bar-row">
                    <div class="is-bar-label">${esc(b.label)}</div>
                    <div class="is-bar-track"><div class="is-bar-midline"></div></div>
                    <div class="is-bar-value">${esc(t('common.no_trades'))}</div>
                </div>`;
        }
        const widthPct = Math.max(0, Math.min(50, (Math.abs(pnl) / maxAbs) * 50)).toFixed(2);
        const cls = pnl >= 0 ? 'is-fill-pos lq-fill-win' : 'is-fill-neg lq-fill-lose';
        return `
            <div class="is-bar-row">
                <div class="is-bar-label">${esc(b.label)}</div>
                <div class="is-bar-track">
                    <div class="is-bar-midline"></div>
                    <div class="is-bar-fill ${cls}" data-bar-pct="${widthPct}"></div>
                </div>
                <div class="is-bar-value">
                    n=${b.trades} · ${esc(fmtUSD(pnl))} · win ${esc((b.win_rate * 100).toFixed(0))}%
                </div>
            </div>`;
    }).join('');
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.is-bar-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function showErr(msg) {
    const el = document.getElementById('lq-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('lq-err').style.display = 'none'; }
