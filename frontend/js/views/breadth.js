// Market breadth — NYSE TICK / TRIN / A-D / Up-Down Vol / Put-Call + regime.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n, t } from '../i18n.js';
import { showToast } from '../toast.js';

let timer = null;

export async function renderBreadth(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.breadth.h1.market_breadth" class="view-title">// MARKET BREADTH</h1>
        <div data-context-scope="market-breadth">
        <p data-i18n="view.breadth.hint.intraday_tape_regime_nyse_tick_instantaneous_up_ti" class="muted small">Intraday tape regime: NYSE TICK (instantaneous up-tick count),
            TRIN (Arms Index — volume bias), Advance-Decline issues, Up-Down volume,
            CBOE Put-Call ratio. Composite score combines all five into a -100..+100
            regime gauge. Polls every 60s.</p>

        <div class="inline-form">
            <button data-i18n="view.breadth.btn.refresh" id="mb-refresh" class="primary"
                    data-tip="view.breadth.tip.refresh" data-shortcut="breadth_refresh" type="button">Refresh now</button>
        </div>

        <div id="bcomp" class="cards"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
        <div id="binds"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.breadth.h2.indicator_chart">Indicator % change snapshot</h2>
            <div id="b-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.breadth.h2.value_chart">Indicator raw values (current level)</h2>
            <div id="b-value-chart" style="width:100%;height:200px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.breadth.h2.regime_guide">Regime guide</h2>
            <table class="trades">
                <thead><tr><th data-i18n="view.breadth.th.indicator">Indicator</th><th data-i18n="view.breadth.th.strong_bull">Strong bull</th><th data-i18n="view.breadth.th.mild_bull">Mild bull</th><th data-i18n="view.breadth.th.neutral">Neutral</th><th data-i18n="view.breadth.th.mild_bear">Mild bear</th><th data-i18n="view.breadth.th.strong_bear">Strong bear</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.breadth.row.nyse_tick">NYSE TICK</td><td class="pos">≥ +800</td><td class="pos">+400..+800</td><td>±400</td><td class="neg">−400..−800</td><td class="neg">≤ −800</td></tr>
                    <tr><td data-i18n="view.breadth.row.nyse_trin">NYSE TRIN</td><td class="pos">≤ 0.5</td><td class="pos">0.5..0.9</td><td>0.9..1.1</td><td class="neg">1.1..2.0</td><td class="neg">≥ 2.0</td></tr>
                    <tr><td data-i18n="view.breadth.row.advance_decline">Advance−Decline</td><td class="pos">≥ +1500</td><td class="pos">+500..+1500</td><td>±500</td><td class="neg">−500..−1500</td><td class="neg">≤ −1500</td></tr>
                    <tr><td data-i18n="view.breadth.row.put_call_ratio">Put-Call ratio</td><td class="neg">≤ 0.6 *</td><td class="pos">0.6..0.8</td><td>0.8..1.0</td><td class="neg">1.0..1.2</td><td class="pos">≥ 1.2 *</td></tr>
                </tbody>
            </table>
            <p data-i18n="view.breadth.hint.put_call_is_a_contrarian_indicator_at_extremes_ver" class="muted small">* Put-Call is a contrarian indicator at extremes — very low PCR = complacency (often near tops), very high PCR = fear (often near bottoms).</p>
        </div>
    `;
    if (timer) clearInterval(timer);
    timer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(timer); timer = null; return; }
        refresh(mount, tok, false);
    }, 60_000);
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#breadth')) { clearInterval(timer); timer = null; }
    }, { once: true });
    const refreshBtn = document.getElementById('mb-refresh');
    if (refreshBtn) refreshBtn.addEventListener('click', () => { void refresh(mount, tok, true); });
    await refresh(mount, tok, false);
}

async function refresh(mount, tok, userInitiated) {
    try {
        const s = await api.breadthSnapshot();
        if (!viewIsCurrent(tok)) return;
        renderComposite(s, mount);
        renderIndicators(s, mount);
        if (userInitiated) showToast(t('view.breadth.toast.refreshed'), { level: 'success' });
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#bcomp');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
        if (userInitiated) showToast(t('view.breadth.toast.api_error'), { level: 'error' });
    }
}

function renderComposite(s, mount) {
    const regCls = s.regime === 'bullish' ? 'pos' : s.regime === 'bearish' ? 'neg' : '';
    const scoreCls = s.composite_score >= 30 ? 'pos' : s.composite_score <= -30 ? 'neg' : '';
    const el = mount.querySelector('#bcomp');
    if (!el) return;
    el.innerHTML = `
        <div class="card"><div class="label" data-i18n="view.breadth.card.composite_score">Composite score</div>
            <div class="value ${scoreCls}">${s.composite_score >= 0 ? '+' : ''}${s.composite_score}</div></div>
        <div class="card"><div class="label" data-i18n="view.breadth.card.regime">Regime</div>
            <div class="value ${regCls}">${s.regime.toUpperCase()}</div></div>
        <div class="card"><div class="label" data-i18n="view.breadth.card.indicators_fired">Indicators fired</div>
            <div class="value">${[s.tick, s.trin, s.addn, s.vold, s.pcr].filter(Boolean).length} / 5</div></div>
        <div class="card"><div class="label" data-i18n="view.breadth.card.updated">Updated</div>
            <div class="value small">${new Date(s.fetched_at).toLocaleTimeString(undefined, { hour12: false })}</div></div>
    `;
    try { applyUiI18n(el); } catch (_) {}
}

function renderIndicators(s, mount) {
    const inds = [s.tick, s.trin, s.addn, s.vold, s.pcr].filter(Boolean);
    const el = mount.querySelector('#binds');
    if (!el) return;
    if (!inds.length) {
        el.innerHTML = '<p data-i18n="view.breadth.hint.no_breadth_tickers_returned_data_try_in_market_hou" class="boot">No breadth tickers returned data — try in market hours.</p>';
        return;
    }
    el.innerHTML = `
        <div class="cards">
            ${inds.map(i => {
                const chCls = i.change_pct >= 0 ? 'pos' : 'neg';
                return `<div class="card" data-context-scope="symbol-row" data-symbol="${esc(i.symbol)}">
                    <div class="label">${esc(i.label)} (${esc(i.symbol)})</div>
                    <div class="value">${fmt(i.value, Math.abs(i.value) < 10 ? 3 : 0)}</div>
                    <div class="small ${chCls}">${i.change_pct >= 0 ? '+' : ''}${i.change_pct.toFixed(2)}%</div>
                    <div class="muted small">${esc(i.interpretation)}</div>
                </div>`;
            }).join('')}
        </div>
    `;
    renderIndicatorChart(inds);
    renderValueChart(inds);
}

function renderValueChart(inds) {
    const el = document.getElementById('b-value-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (inds || []).filter(i => Number.isFinite(Number(i.value)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.breadth.empty_value_chart">${esc(t('view.breadth.empty_value_chart'))}</div>`;
        return;
    }
    const labels = valid.map(i => i.symbol);
    const ys = valid.map(i => Number(i.value));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.breadth.chart.indicator_idx') },
            { label: t('view.breadth.chart.value'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderIndicatorChart(inds) {
    const el = document.getElementById('b-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (inds || []).filter(i => Number.isFinite(Number(i.change_pct)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.breadth.empty_chart">${esc(t('view.breadth.empty_chart'))}</div>`;
        return;
    }
    const labels = valid.map(i => i.symbol);
    const ys = valid.map(i => Number(i.change_pct));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.breadth.chart.indicator_idx') },
            { label: t('view.breadth.chart.change_pct'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.breadth.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}
