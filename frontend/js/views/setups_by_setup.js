// Setups-by-setup view — rolls trades up by setup tag and ranks them.
// Per-setup: trades count, wins/losses/scratches, win-rate, profit
// factor, avg R, expectancy. Sorted by net_pnl DESC so winning setups
// float to the top.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSetupTradeBlob, validateInputs, buildBody, localAnalyze,
    dec, setupBadge, makeDemoRows,
    fmtUSD, fmtUSDSigned, fmtPct, fmtPF, fmtR,
} from '../_setups_by_setup_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { rows: makeDemoRows('mixed') };

export async function renderSetupsBySetup(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.setups_by_setup.h1.setups_ranked_by_pandl" class="view-title">// SETUPS — RANKED BY P&amp;L</h1>

        <div class="chart-panel">
            <h2><span data-i18n="view.setups_by_setup.h2.trades">Trades (per line:</span> <code>setup net_pnl [risk_amount]</code><span data-i18n="view.setups_by_setup.h2.trades_suffix">; "-" = untagged)</span></h2>
            <textarea id="sbs-blob" rows="10" placeholder="orb 500 100&#10;abcd -150 100&#10;- 999 100   # untagged, excluded" data-i18n-placeholder="view.setups_by_setup.placeholder.blob" data-tip="view.setups_by_setup.tip.blob">${esc(rowsToBlob(state.rows))}</textarea>
            <div class="inline-form">
                <button data-i18n="view.setups_by_setup.btn.analyze" id="sbs-run" class="primary" type="button" data-tip="view.setups_by_setup.tip.run" data-shortcut="setups_by_setup_run">Analyze</button>
                <button data-i18n="view.setups_by_setup.btn.demo_3_setups_mixed" id="sbs-demo-mixed"    class="secondary" type="button" data-tip="view.setups_by_setup.tip.demo_mixed">Demo: 3 setups mixed</button>
                <button data-i18n="view.setups_by_setup.btn.demo_single_winner" id="sbs-demo-winner"   class="secondary" type="button" data-tip="view.setups_by_setup.tip.demo_winner">Demo: single winner</button>
                <button data-i18n="view.setups_by_setup.btn.demo_single_loser" id="sbs-demo-loser"    class="secondary" type="button" data-tip="view.setups_by_setup.tip.demo_loser">Demo: single loser</button>
                <button data-i18n="view.setups_by_setup.btn.demo_with_untagged" id="sbs-demo-untag"    class="secondary" type="button" data-tip="view.setups_by_setup.tip.demo_untag">Demo: with untagged</button>
                <button data-i18n="view.setups_by_setup.btn.demo_all_scratches" id="sbs-demo-scratch"  class="secondary" type="button" data-tip="view.setups_by_setup.tip.demo_scratch">Demo: all scratches</button>
            </div>
            <p data-i18n="view.setups_by_setup.hint.risk_amount_enables_r_multiple_net_p_l_0_win_0_los" class="muted">Risk amount enables R-multiple. Net P&L &gt; 0 = win, &lt; 0 = loss, == 0 = scratch. Untagged trades ("-") are sent to the backend but excluded from any setup bucket — matches backend "no setup tag → not in the catalog".</p>
        </div>

        <div id="sbs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.setups_by_setup.h2.per_setup_leaderboard">Per-setup leaderboard</h2>
            <div id="sbs-stats"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.setups_by_setup.h2.net_pnl_chart">Net P&L per setup</h2>
            <div id="sbs-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.setups_by_setup.h2.wlr_chart">Wins / losses / win-rate per setup</h2>
            <div id="sbs-wlr-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.setups_by_setup.h2.pf_chart">Profit factor per setup</h2>
            <div id="sbs-pf-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.setups_by_setup.h2.avgr_chart">Average R-multiple per setup</h2>
            <div id="sbs-avgr-chart" style="width:100%;height:200px"></div>
        </div>

        <div id="sbs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.rows = makeDemoRows(kind);
        document.getElementById('sbs-blob').value = rowsToBlob(state.rows);
    };
    document.getElementById('sbs-demo-mixed').addEventListener('click',   () => loadDemo('mixed'));
    document.getElementById('sbs-demo-winner').addEventListener('click',  () => loadDemo('single-winner'));
    document.getElementById('sbs-demo-loser').addEventListener('click',   () => loadDemo('single-loser'));
    document.getElementById('sbs-demo-untag').addEventListener('click',   () => loadDemo('with-untagged'));
    document.getElementById('sbs-demo-scratch').addEventListener('click', () => loadDemo('all-scratches'));
    document.getElementById('sbs-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function rowsToBlob(rows) {
    return rows.map(r => {
        const risk = r.risk_amount != null ? ` ${r.risk_amount}` : '';
        return `${r.setup} ${r.net_pnl}${risk}`;
    }).join('\n');
}

function readInputs() {
    const parsed = parseSetupTradeBlob(document.getElementById('sbs-blob').value);
    if (parsed.errors.length) {
        showErr(t("common.error.parse_errors", { summary: parsed.errors.slice(0, 3).map(e => `[] `).join("; ") }));
        showToast(t('view.setups_by_setup.toast.parse_error'), { level: 'warning' });
        return;
    }
    hideErr();
    state.rows = parsed.rows;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.rows);
    if (err) { showErr(err); showToast(t('view.setups_by_setup.toast.invalid'), { level: 'warning' }); return; }
    const local = localAnalyze(state.rows);
    renderSummary(local, true);
    renderStats(local, true);
    renderNetPnlChart(local);
    renderWlrChart(local);
    renderPfChart(local);
    renderAvgRChart(local);
    let resp;
    try {
        resp = await api.setupsBySetup(buildBody(state.rows));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.setups_by_setup.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    // Backend returns Decimals as strings — coerce to numbers.
    const normalized = resp.map(s => ({
        ...s,
        net_pnl: dec(s.net_pnl), gross_pnl: dec(s.gross_pnl), fees: dec(s.fees),
        avg_pnl: dec(s.avg_pnl), avg_win: dec(s.avg_win), avg_loss: dec(s.avg_loss),
        expectancy: dec(s.expectancy),
        largest_win: dec(s.largest_win), largest_loss: dec(s.largest_loss),
    }));
    renderSummary(normalized, false);
    renderStats(normalized, false);
    renderNetPnlChart(normalized);
    renderWlrChart(normalized);
    renderPfChart(normalized);
    renderAvgRChart(normalized);
    showToast(t('view.setups_by_setup.toast.analyzed', { n: normalized.length }), { level: 'success' });
}

function renderPfChart(stats) {
    const el = document.getElementById('sbs-pf-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (stats || []).filter(s => Number.isFinite(Number(s.profit_factor)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.setups_by_setup.empty_pf_chart">${esc(t('view.setups_by_setup.empty_pf_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.profit_factor) - Number(a.profit_factor));
    const labels = rows.map(s => s.setup);
    const xs = labels.map((_, i) => i + 1);
    const winY  = rows.map(s => Number(s.profit_factor) >= 1 ? Number(s.profit_factor) : null);
    const loseY = rows.map(s => Number(s.profit_factor) <  1 ? Number(s.profit_factor) : null);
    const one   = xs.map(() => 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.setups_by_setup.chart.setup') },
            { label: t('view.setups_by_setup.chart.pf_good'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.setups_by_setup.chart.pf_bad'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.setups_by_setup.chart.pf_one'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, winY, loseY, one], el);
}

function renderWlrChart(stats) {
    const el = document.getElementById('sbs-wlr-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (stats || []).filter(s =>
        Number.isFinite(Number(s.wins)) && Number.isFinite(Number(s.losses)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.setups_by_setup.empty_wlr_chart">${esc(t('view.setups_by_setup.empty_wlr_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => (Number(b.wins) + Number(b.losses)) - (Number(a.wins) + Number(a.losses)));
    const labels = rows.map(s => s.setup);
    const xs = labels.map((_, i) => i + 1);
    const wins   = rows.map(s => Number(s.wins));
    const losses = rows.map(s => Number(s.losses));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.setups_by_setup.chart.setup') },
            { label: t('view.setups_by_setup.chart.wins'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.setups_by_setup.chart.losses'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, wins, losses], el);
}

function renderAvgRChart(stats) {
    const el = document.getElementById('sbs-avgr-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (stats || []).filter(s => Number.isFinite(Number(s.avg_r)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.setups_by_setup.empty_avgr_chart">${esc(t('view.setups_by_setup.empty_avgr_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.avg_r) - Number(a.avg_r));
    const labels = rows.map(s => s.setup);
    const xs = labels.map((_, i) => i + 1);
    const posY = rows.map(s => Number(s.avg_r) >= 0 ? Number(s.avg_r) : null);
    const negY = rows.map(s => Number(s.avg_r) <  0 ? Number(s.avg_r) : null);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.setups_by_setup.chart.setup') },
            { label: t('view.setups_by_setup.chart.avgr_pos'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.setups_by_setup.chart.avgr_neg'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.setups_by_setup.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, posY, negY, zero], el);
}

function renderSummary(stats, pending) {
    const local = localAnalyze(state.rows);
    const parity = statsArrEq(stats, local);
    const totalTrades = stats.reduce((a, s) => a + s.trades, 0);
    const totalNet    = stats.reduce((a, s) => a + s.net_pnl, 0);
    const totalWins   = stats.reduce((a, s) => a + s.wins, 0);
    const totalLosses = stats.reduce((a, s) => a + s.losses, 0);
    const untagged    = state.rows.filter(r => r.setup === '-' || r.setup === '').length;
    const winRate     = totalTrades > 0 ? totalWins / totalTrades : 0;
    const best  = stats.length > 0 ? stats[0] : null;
    const worst = stats.length > 0 ? stats[stats.length - 1] : null;
    document.getElementById('sbs-summary').innerHTML = [
        card(t('view.setups_by_setup.card.setups'),         String(stats.length) + (pending ? t('common.suffix.local') : '')),
        card(t('view.setups_by_setup.card.tagged_trades'),  String(totalTrades)),
        card(t('view.setups_by_setup.card.untagged_skipped'), String(untagged),
            untagged > 0 ? 'neg' : ''),
        card(t('view.setups_by_setup.card.total_net_p_l'),  fmtUSDSigned(totalNet),
            totalNet >= 0 ? 'pos' : 'neg'),
        card(t('view.setups_by_setup.card.total_win_rate'), fmtPct(winRate),
            winRate >= 0.5 ? 'pos' : 'neg'),
        card(t('view.setups_by_setup.card.wins_losses'),  `${totalWins} / ${totalLosses}`,
            totalWins > totalLosses ? 'pos' : 'neg'),
        card(t('view.setups_by_setup.card.best_setup'),     best ? `${best.setup}: ${fmtUSDSigned(best.net_pnl)}` : '—',
            best && best.net_pnl >= 0 ? 'pos' : 'neg'),
        card(t('view.setups_by_setup.card.worst_setup'),    worst ? `${worst.setup}: ${fmtUSDSigned(worst.net_pnl)}` : '—',
            worst && worst.net_pnl >= 0 ? 'pos' : 'neg'),
        card(t('view.setups_by_setup.card.local_parity'),   parity ? t('common.ok') : t('common.diverged'), parity ? 'pos' : 'neg'),
    ].join('');
}

function statsArrEq(a, b) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        if (a[i].setup !== b[i].setup) return false;
        if (Math.abs(a[i].net_pnl - b[i].net_pnl) > 1e-6) return false;
        if (a[i].trades !== b[i].trades) return false;
        if (a[i].wins !== b[i].wins) return false;
        if (a[i].losses !== b[i].losses) return false;
    }
    return true;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderNetPnlChart(stats) {
    const el = document.getElementById('sbs-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (stats || []).filter(s => Number.isFinite(Number(s.net_pnl)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.setups_by_setup.empty_chart">${esc(t('view.setups_by_setup.empty_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.net_pnl) - Number(a.net_pnl));
    const labels = rows.map(s => s.setup);
    const xs = labels.map((_, i) => i + 1);
    const winY  = rows.map(s => Number(s.net_pnl) >= 0 ? Number(s.net_pnl) : null);
    const loseY = rows.map(s => Number(s.net_pnl) <  0 ? Number(s.net_pnl) : null);
    const zero  = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.setups_by_setup.chart.setup') },
            { label: t('view.setups_by_setup.chart.win'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.setups_by_setup.chart.lose'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.setups_by_setup.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, winY, loseY, zero], el);
}

function renderStats(stats) {
    const wrap = document.getElementById('sbs-stats');
    if (!stats.length) { wrap.innerHTML = `<div class="muted" data-i18n="view.setups_by_setup.empty.trades">No tagged trades.</div>`; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.setups_by_setup.th.setup">Setup</th><th data-i18n="view.setups_by_setup.th.verdict">Verdict</th>
                <th data-i18n="view.setups_by_setup.th.trades">Trades</th><th data-i18n="view.setups_by_setup.th.w_l_s">W/L/S</th><th data-i18n="view.setups_by_setup.th.win">Win%</th>
                <th data-i18n="view.setups_by_setup.th.net_p_l">Net P&L</th><th data-i18n="view.setups_by_setup.th.expectancy">Expectancy</th><th data-i18n="view.setups_by_setup.th.avg_win">Avg win</th><th data-i18n="view.setups_by_setup.th.avg_loss">Avg loss</th>
                <th data-i18n="view.setups_by_setup.th.profit_factor">Profit factor</th><th data-i18n="view.setups_by_setup.th.avg_r">Avg R</th>
                <th data-i18n="view.setups_by_setup.th.largest_win">Largest win</th><th data-i18n="view.setups_by_setup.th.largest_loss">Largest loss</th>
            </tr></thead>
            <tbody>
                ${stats.map((s, i) => {
                    const badge = setupBadge(s);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td><strong>${esc(s.setup)}</strong></td>
                        <td class="${badge.cls}">${esc(badge.label)}</td>
                        <td>${s.trades}</td>
                        <td>${s.wins}/${s.losses}/${s.scratches}</td>
                        <td class="${s.win_rate >= 0.5 ? 'pos' : 'neg'}">${esc(fmtPct(s.win_rate))}</td>
                        <td class="${s.net_pnl >= 0 ? 'pos' : 'neg'}">${esc(fmtUSDSigned(s.net_pnl))}</td>
                        <td class="${s.expectancy >= 0 ? 'pos' : 'neg'}">${esc(fmtUSDSigned(s.expectancy))}</td>
                        <td class="pos">${esc(fmtUSDSigned(s.avg_win))}</td>
                        <td class="neg">${esc(fmtUSDSigned(s.avg_loss))}</td>
                        <td class="${s.profit_factor >= 1 ? 'pos' : 'neg'}">${esc(fmtPF(s.profit_factor))}</td>
                        <td class="${s.avg_r >= 0 ? 'pos' : 'neg'}">${esc(fmtR(s.avg_r))}</td>
                        <td class="pos">${esc(fmtUSD(s.largest_win))}</td>
                        <td class="neg">${esc(fmtUSDSigned(s.largest_loss))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('sbs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('sbs-err').style.display = 'none'; }
