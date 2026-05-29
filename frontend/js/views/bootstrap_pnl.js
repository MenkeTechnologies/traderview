// Bootstrap P&L view — non-parametric trade-resample CIs.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_RESAMPLES, DEFAULT_SEED,
    parseTradesBlob, tradesToBlob, validateInputs, buildBody, localBootstrap,
    probBadge, ciBadge, summarizeTrades,
    makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtPct, fmtInt,
} from '../_bootstrap_pnl_inputs.js';

let state = { ...makeDemoInput('winning-strategy') };

export async function renderBootstrapPnl(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.boot_pnl.h1.title" class="view-title">// BOOTSTRAP P&L CI</h1>

        <div class="chart-panel" data-context-scope="bootstrap-pnl">
            <h2 data-i18n="view.boot_pnl.h2.trades">Trade P&amp;Ls
                <small data-i18n="view.boot_pnl.h2.trades_hint" class="muted">(one per token; "$50" or "(50)" for losses accepted)</small></h2>
            <textarea id="bp-blob" rows="6"
                      data-tip="view.boot_pnl.tip.trades"
                      placeholder="50, -30, 50, 50, -30, ...">${esc(tradesToBlob(state.trade_pnls))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.boot_pnl.label.resamples">Resamples</span>
                    <input id="bp-resamples" type="number" step="1" min="100" value="${state.n_resamples}"></label>
                <label><span data-i18n="view.boot_pnl.label.seed">Seed</span>
                    <input id="bp-seed" type="number" step="1" value="${state.seed}"></label>
                <button data-i18n="view.boot_pnl.btn.compute" id="bp-run" class="primary"
                        data-tip="view.boot_pnl.tip.compute" type="button">Resample</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.boot_pnl.btn.demo_winning"  id="bp-demo-win"   class="secondary" type="button">Demo: winning strategy</button>
                <button data-i18n="view.boot_pnl.btn.demo_losing"   id="bp-demo-loss"  class="secondary" type="button">Demo: losing strategy</button>
                <button data-i18n="view.boot_pnl.btn.demo_hivar"    id="bp-demo-hi"    class="secondary" type="button">Demo: high variance</button>
                <button data-i18n="view.boot_pnl.btn.demo_lovar"    id="bp-demo-lo"    class="secondary" type="button">Demo: low variance grid</button>
                <button data-i18n="view.boot_pnl.btn.demo_winall"   id="bp-demo-wa"    class="secondary" type="button">Demo: all winners</button>
                <button data-i18n="view.boot_pnl.btn.demo_lossall"  id="bp-demo-la"    class="secondary" type="button">Demo: all losers</button>
                <button data-i18n="view.boot_pnl.btn.demo_lumpy"    id="bp-demo-lumpy" class="secondary" type="button">Demo: lumpy-tail (95% +$10, 5% −$500)</button>
                <button data-i18n="view.boot_pnl.btn.demo_few"      id="bp-demo-few"   class="secondary" type="button">Demo: few trades (8)</button>
            </div>
            <p data-i18n="view.boot_pnl.hint.about" class="muted">Resamples per-trade P&amp;L with replacement. Reports mean / median / 90% CI / 95% CI / Pr(total > 0). iid bootstrap is appropriate for trade-level data where trades are independent. For serially-dependent returns use block bootstrap.</p>
        </div>

        <div id="bp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.boot_pnl.h2.cards">Confidence intervals</h2>
            <div id="bp-ci"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.boot_pnl.h2.table">Per-trade summary</h2>
            <div id="bp-table"></div>
        </div>

        <div id="bp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bp-blob').value     = tradesToBlob(state.trade_pnls);
        document.getElementById('bp-resamples').value = state.n_resamples;
        document.getElementById('bp-seed').value     = String(state.seed);
    };
    document.getElementById('bp-demo-win').addEventListener('click',  () => { loadDemo('winning-strategy'); void compute(tok); });
    document.getElementById('bp-demo-loss').addEventListener('click', () => { loadDemo('losing-strategy');  void compute(tok); });
    document.getElementById('bp-demo-hi').addEventListener('click',   () => { loadDemo('high-variance');    void compute(tok); });
    document.getElementById('bp-demo-lo').addEventListener('click',   () => { loadDemo('low-variance');     void compute(tok); });
    document.getElementById('bp-demo-wa').addEventListener('click',   () => { loadDemo('all-winners');      void compute(tok); });
    document.getElementById('bp-demo-la').addEventListener('click',   () => { loadDemo('all-losers');       void compute(tok); });
    document.getElementById('bp-demo-lumpy').addEventListener('click', () => { loadDemo('lumpy-tail');     void compute(tok); });
    document.getElementById('bp-demo-few').addEventListener('click',  () => { loadDemo('few-trades');       void compute(tok); });
    document.getElementById('bp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseTradesBlob(document.getElementById('bp-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.boot_pnl.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.trade_pnls = p.trade_pnls;
    const resamples = parseInt(document.getElementById('bp-resamples').value, 10);
    const seedInput = document.getElementById('bp-seed').value;
    state.n_resamples = Number.isInteger(resamples) && resamples >= 100 ? resamples : DEFAULT_RESAMPLES;
    let seed;
    try {
        seed = BigInt(seedInput);
    } catch {
        seed = DEFAULT_SEED;
    }
    state.seed = seed;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localBootstrap(state.trade_pnls, state.n_resamples, state.seed);
    if (!local) { showErr(t('view.boot_pnl.err.degenerate')); return; }
    renderSummary(local, true);
    renderCi(local);
    renderTable();
    let resp;
    try {
        resp = await api.anlyBootstrapPnl(buildBody(state));
    } catch (e) {
        showErr(`${t('view.boot_pnl.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.boot_pnl.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderCi(resp);
    renderTable();
}

function renderSummary(report, pending) {
    const local = localBootstrap(state.trade_pnls, state.n_resamples, state.seed);
    const parityOk = !!local
        && Math.abs(local.mean_total_pnl - report.mean_total_pnl) < 1e-6
        && Math.abs(local.probability_positive - report.probability_positive) < 1e-9
        && local.n_resamples === report.n_resamples;
    const pBadge = probBadge(report.probability_positive);
    const cBadge = ciBadge(report);
    const tStat = summarizeTrades(state.trade_pnls);
    const localTag = pending ? ` (${t('view.boot_pnl.tag.local')})` : '';
    document.getElementById('bp-summary').innerHTML = [
        card(t('view.boot_pnl.card.verdict'),   t(pBadge.key) + localTag, pBadge.cls),
        card(t('view.boot_pnl.card.ci_width'),  t(cBadge.key), cBadge.cls),
        card(t('view.boot_pnl.card.trades'),    fmtInt(report.n_trades)),
        card(t('view.boot_pnl.card.resamples'), fmtInt(report.n_resamples)),
        card(t('view.boot_pnl.card.win_rate'),  fmtPct(tStat.win_rate)),
        card(t('view.boot_pnl.card.expectancy'), fmtUSDSigned(tStat.mean)),
        card(t('view.boot_pnl.card.mean'),      fmtUSDSigned(report.mean_total_pnl),
             report.mean_total_pnl > 0 ? 'pos' : report.mean_total_pnl < 0 ? 'neg' : ''),
        card(t('view.boot_pnl.card.median'),    fmtUSDSigned(report.median_total_pnl)),
        card(t('view.boot_pnl.card.prob_pos'),  fmtPct(report.probability_positive), pBadge.cls),
        card(t('view.boot_pnl.card.parity'),
             parityOk ? t('view.boot_pnl.tag.ok') : t('view.boot_pnl.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderCi(report) {
    const wrap = document.getElementById('bp-ci');
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.boot_pnl.col.ci">Interval</th>
                <th data-i18n="view.boot_pnl.col.lo">Lower</th>
                <th data-i18n="view.boot_pnl.col.hi">Upper</th>
                <th data-i18n="view.boot_pnl.col.width">Width</th>
            </tr></thead>
            <tbody>
                <tr>
                    <td data-i18n="view.boot_pnl.row.ci90"><strong>90% CI</strong></td>
                    <td class="${report.pnl_5th_percentile < 0 ? 'neg' : 'pos'}">${esc(fmtUSDSigned(report.pnl_5th_percentile))}</td>
                    <td class="${report.pnl_95th_percentile < 0 ? 'neg' : 'pos'}">${esc(fmtUSDSigned(report.pnl_95th_percentile))}</td>
                    <td>${esc(fmtUSD(report.pnl_95th_percentile - report.pnl_5th_percentile))}</td>
                </tr>
                <tr>
                    <td data-i18n="view.boot_pnl.row.ci95"><strong>95% CI</strong></td>
                    <td class="${report.pnl_2_5th_percentile < 0 ? 'neg' : 'pos'}">${esc(fmtUSDSigned(report.pnl_2_5th_percentile))}</td>
                    <td class="${report.pnl_97_5th_percentile < 0 ? 'neg' : 'pos'}">${esc(fmtUSDSigned(report.pnl_97_5th_percentile))}</td>
                    <td>${esc(fmtUSD(report.pnl_97_5th_percentile - report.pnl_2_5th_percentile))}</td>
                </tr>
            </tbody>
        </table>
    `;
}

function renderTable() {
    const wrap = document.getElementById('bp-table');
    const trades = state.trade_pnls;
    if (!trades.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.boot_pnl.empty">${esc(t('view.boot_pnl.empty'))}</div>`;
        return;
    }
    const s = summarizeTrades(trades);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.boot_pnl.col.metric">Metric</th>
                <th data-i18n="view.boot_pnl.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.boot_pnl.row.count">Trades</td>           <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.boot_pnl.row.sum">Total realized</td>      <td class="${s.sum > 0 ? 'pos' : s.sum < 0 ? 'neg' : ''}">${esc(fmtUSDSigned(s.sum))}</td></tr>
                <tr><td data-i18n="view.boot_pnl.row.mean">Mean per trade</td>     <td class="${s.mean > 0 ? 'pos' : s.mean < 0 ? 'neg' : ''}">${esc(fmtUSDSigned(s.mean))}</td></tr>
                <tr><td data-i18n="view.boot_pnl.row.wins">Wins</td>                <td class="pos">${fmtInt(s.wins)}</td></tr>
                <tr><td data-i18n="view.boot_pnl.row.losses">Losses</td>            <td class="neg">${fmtInt(s.losses)}</td></tr>
                <tr><td data-i18n="view.boot_pnl.row.winrate">Win rate</td>         <td>${esc(fmtPct(s.win_rate))}</td></tr>
                <tr><td data-i18n="view.boot_pnl.row.max_win">Best trade</td>       <td class="pos">${esc(fmtUSDSigned(s.max_win))}</td></tr>
                <tr><td data-i18n="view.boot_pnl.row.max_loss">Worst trade</td>     <td class="neg">${esc(fmtUSDSigned(s.max_loss))}</td></tr>
            </tbody>
        </table>
    `;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('bp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bp-err').style.display = 'none'; }
