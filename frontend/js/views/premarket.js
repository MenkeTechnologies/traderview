// Pre-market futures dashboard — index futures, commodities, crypto, FX.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let timer = null;

export async function renderPremarket(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.premarket.h1.pre_market_overnight" class="view-title">// PRE-MARKET / OVERNIGHT</h1>
        <p data-i18n="view.premarket.hint.cross_asset_overnight_tape_index_futures_commoditi" class="muted small">Cross-asset overnight tape: index futures, commodities, crypto, FX.
            Each gap is normalized by 20-day ATR — magnitudes above 1.0× ATR are statistically
            significant moves vs the security's own recent volatility. High-importance economic
            releases scheduled for today appear at the bottom. Refreshes every 30s.</p>

        <div id="pmEvents"></div>
        <div id="pmContent" class="cards"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
        <div class="chart-panel">
            <h2 data-i18n="view.premarket.h2.change_chart">Overnight change % across contracts</h2>
            <div id="pm-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.premarket.h2.atr_chart">ATR-normalized move (×ATR) across contracts</h2>
            <div id="pm-atr-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.premarket.hint.atr" class="muted small">Overnight move as a multiple of the security's own 20-day ATR. ≥1.0× = statistically significant. Reveals true-significance moves that raw % hides (a 0.3% FX move at 3× ATR is bigger news than a 5% crypto move at 0.4× ATR).</p>
        </div>
    `;
    if (timer) clearInterval(timer);
    timer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(timer); timer = null; return; }
        refresh(mount, tok);
    }, 30_000);
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#premarket')) { clearInterval(timer); timer = null; }
    }, { once: true });
    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    try {
        const s = await api.premarketSnapshot();
        if (!viewIsCurrent(tok)) return;
        renderGroups(s.contracts, mount);
        renderEvents(s.today_events, s.fetched_at, mount);
        renderChangeChart(s.contracts);
        renderAtrChart(s.contracts);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#pmContent');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderGroups(contracts, mount) {
    const groups = new Map();
    for (const c of contracts) {
        if (!groups.has(c.group)) groups.set(c.group, []);
        groups.get(c.group).push(c);
    }
    const out = [];
    for (const [grp, rows] of groups) {
        out.push(`<h2 class="view-title" style="margin-top:1rem;">// ${esc(grp).toUpperCase()}</h2>`);
        out.push(`<div class="cards">${rows.map(card).join('')}</div>`);
    }
    const el = mount.querySelector('#pmContent');
    if (el) el.innerHTML = out.join('');
}

function card(c) {
    if (!c.price) {
        return `<div class="card" data-context-scope="symbol-row" data-symbol="${esc(c.symbol)}">
            <div class="label">${esc(c.symbol)}</div>
            <div class="muted small">${esc(t('common.no_data'))}</div></div>`;
    }
    const ch = c.change_pct;
    const chCls = ch == null ? '' : (ch >= 0 ? 'pos' : 'neg');
    const chTxt = ch == null ? '—' : `${ch >= 0 ? '+' : ''}${ch.toFixed(2)}%`;
    const atrTxt = c.atr_pct == null ? '—' : `${c.atr_pct.toFixed(2)}%`;
    let magTxt = '—', magCls = '';
    if (c.atr_multiple != null) {
        magTxt = `${c.atr_multiple.toFixed(2)}× ATR`;
        if (c.atr_multiple >= 1.5) magCls = 'neg';
        else if (c.atr_multiple >= 1.0) magCls = 'warn';
    }
    const rng = (c.day_high != null && c.day_low != null)
        ? `<div class="muted small">${esc(t('common.range_label', { low: fmt(c.day_low), high: fmt(c.day_high) }))}</div>`
        : '';
    const ms = c.market_state ? `<div class="muted small">${esc(c.market_state.toLowerCase())}</div>` : '';
    return `<div class="card" data-context-scope="symbol-row" data-symbol="${esc(c.symbol)}">
        <div class="label">${esc(c.label)} (${esc(c.symbol)})</div>
        <div class="value">${fmt(c.price, c.price < 10 ? 4 : 2)}</div>
        <div class="small ${chCls}">${chTxt}</div>
        <div class="muted small">${esc(t('view.premarket.tile.atr20', { atr: atrTxt }))}</div>
        <div class="small ${magCls}">${magTxt}</div>
        ${rng}${ms}
    </div>`;
}

function renderChangeChart(contracts) {
    const el = document.getElementById('pm-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (contracts || []).filter(c => Number.isFinite(Number(c.change_pct)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.premarket.empty_chart">${esc(t('view.premarket.empty_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => Number(b.change_pct) - Number(a.change_pct));
    const labels = valid.map(c => c.symbol);
    const ys = valid.map(c => Number(c.change_pct));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.premarket.chart.contract_idx') },
            { label: t('view.premarket.chart.change_pct'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.premarket.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderAtrChart(contracts) {
    const el = document.getElementById('pm-atr-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (contracts || []).filter(c => Number.isFinite(Number(c.atr_multiple)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.premarket.empty_atr_chart">${esc(t('view.premarket.empty_atr_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => Math.abs(Number(b.atr_multiple)) - Math.abs(Number(a.atr_multiple)));
    const labels = valid.map(c => c.symbol);
    const ys = valid.map(c => Number(c.atr_multiple));
    const xs = labels.map((_, i) => i + 1);
    const one = xs.map(() => 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.premarket.chart.contract_idx') },
            { label: t('view.premarket.chart.atr_multiple'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.premarket.chart.atr_threshold'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50,
              values: (_u, splits) => splits.map(v => v.toFixed(1) + '×') },
        ],
        legend: { show: true },
    }, [xs, ys, one], el);
}

function renderEvents(events, fetched, mount) {
    const el = mount.querySelector('#pmEvents');
    if (!el) return;
    if (!events || !events.length) {
        el.innerHTML = `
            <div class="chart-panel">
                <h2 data-i18n="view.premarket.h2.today_s_high_impact_releases">Today's high-impact releases</h2>
                <p class="muted small">${esc(t('view.premarket.hint.none_today', { time: new Date(fetched).toLocaleTimeString(undefined, { hour12: false }) }))}</p>
            </div>`;
        return;
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2>${esc(t('view.premarket.h2.high_impact', { count: events.length }))}</h2>
            <table class="trades">
                <thead><tr><th data-i18n="view.premarket.th.time_et">Time (ET)</th><th data-i18n="view.premarket.th.event">Event</th><th data-i18n="view.premarket.th.category">Category</th><th data-i18n="view.premarket.th.source">Source</th></tr></thead>
                <tbody>
                    ${events.map(e => `<tr>
                        <td>${esc(e.when_et.split('T')[1].slice(0, 5))}</td>
                        <td>${esc(e.name)}</td>
                        <td>${esc(e.category)}</td>
                        <td class="small muted">${esc(e.source)}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">${esc(t('view.premarket.hint.updated', { time: new Date(fetched).toLocaleTimeString(undefined, { hour12: false }) }))}</p>
        </div>
    `;
}
