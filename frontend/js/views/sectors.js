import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

export async function renderSectors(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.sectors.h1.sector_strength" class="view-title">// SECTOR STRENGTH</h1>
        <p data-i18n="view.sectors.hint.11_spdr_sector_etfs_ranked_by_today_s_change_and_r" class="muted small">11 SPDR sector ETFs ranked by today's % change and relative strength vs SPY (ZenBot-style).</p>
        <div id="sec"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
    `;
    try {
        const rows = await api.sectors();
        if (!viewIsCurrent(tok)) return;
        const max = Math.max(1, ...rows.map(r => Math.abs(Number(r.change_pct))));
        const secEl = mount.querySelector('#sec');
        if (!secEl) return;
        secEl.innerHTML = `
            <div class="chart-panel">
                <table class="trades">
                    <thead><tr><th>#</th><th data-i18n="view.sectors.th.sector">Sector</th><th data-i18n="view.sectors.th.etf">ETF</th><th data-i18n="view.sectors.th.price">Price</th>
                        <th data-i18n="view.sectors.th.change">Change %</th><th data-i18n="view.sectors.th.rs_vs_spy">RS vs SPY</th><th data-i18n="view.sectors.th.bar">Bar</th></tr></thead>
                    <tbody>${rows.map((s, i) => {
                        const ch = Number(s.change_pct);
                        const rs = s.rs_vs_spy != null ? Number(s.rs_vs_spy) : null;
                        const cls = ch >= 0 ? 'pos' : 'neg';
                        const width = Math.min(100, (Math.abs(ch) / max) * 100);
                        return `<tr>
                            <td>${i + 1}</td>
                            <td>${esc(s.label)}</td>
                            <td><a href="#research/${encodeURIComponent(s.sector)}">${esc(s.sector)}</a></td>
                            <td>${fmt(s.price)}</td>
                            <td class="${cls}">${ch >= 0 ? '+' : ''}${ch.toFixed(2)}%</td>
                            <td class="${rs != null && rs >= 0 ? 'pos' : 'neg'}">${rs != null ? (rs >= 0 ? '+' : '') + rs.toFixed(2) + '%' : '—'}</td>
                            <td><div class="sector-bar ${cls}" style="width:${width}%"></div></td>
                        </tr>`;
                    }).join('')}</tbody>
                </table>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.sectors.h2.change_chart">Today's change % by sector</h2>
                <div id="sec-chart" style="width:100%;height:240px"></div>
            </div>
        `;
        renderChangeChart(rows);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const secEl = mount.querySelector('#sec');
        if (secEl) secEl.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderChangeChart(rows) {
    const el = document.getElementById('sec-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (rows || []).filter(r => Number.isFinite(Number(r.change_pct)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.sectors.empty_chart">${esc(t('view.sectors.empty_chart'))}</div>`;
        return;
    }
    const labels = valid.map(r => r.sector);
    const change = valid.map(r => Number(r.change_pct));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.sectors.chart.sector_idx') },
            { label: t('view.sectors.chart.change_pct'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.sectors.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, change, zero], el);
}
