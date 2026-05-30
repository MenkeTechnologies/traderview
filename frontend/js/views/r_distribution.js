// R-multiple distribution: histogram + SQN + per-tag breakdown.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n, t } from '../i18n.js';

export async function renderRDist(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.r_distribution.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">// R-MULTIPLE DISTRIBUTION — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p data-i18n="view.r_distribution.hint.r_net_pnl_risk_amount_per_trade_requires_risk_amou" class="muted small">R = net_pnl ÷ risk_amount per trade — requires risk_amount set
            on entry. Histogram bins 0.5R from -5R to +5R with tails clamped to the edges.
            SQN = √N × mean(R) ÷ stdev(R) per Van Tharp: under 1.6 poor, 1.6-1.9 below average,
            2.0-2.4 average, 2.5-2.9 good, 3.0-5.0 excellent, &gt;5 suspect (likely curve-fit).</p>

        <div id="r-out"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
    `;
    try {
        const r = await api.rDistribution(acct.id);
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#r-out');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function render(r, mount) {
    const s = r.stats;
    const sqnCls = s.sqn >= 2.5 ? 'pos' : s.sqn >= 1.6 ? '' : 'neg';
    const exCls  = s.mean_r >= 0 ? 'pos' : 'neg';
    const el = mount.querySelector('#r-out');
    if (!el) return;
    el.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.r_distribution.card.samples">Samples</div>
                <div class="value">${s.samples}</div>
                ${r.skipped_no_risk > 0 ? `<div class="small muted">${r.skipped_no_risk} skipped (no risk_amount)</div>` : ''}
            </div>
            <div class="card"><div class="label" data-i18n="view.r_distribution.card.expectancy">Expectancy (mean R)</div>
                <div class="value ${exCls}">${(s.mean_r >= 0 ? '+' : '') + s.mean_r.toFixed(3)}R</div></div>
            <div class="card"><div class="label" data-i18n="view.r_distribution.card.sqn">SQN</div>
                <div class="value ${sqnCls}">${s.sqn.toFixed(2)}</div>
                <div class="small muted">${esc(s.sqn_grade)}</div></div>
            <div class="card"><div class="label" data-i18n="view.r_distribution.card.win_rate">Win rate</div>
                <div class="value">${(s.win_rate * 100).toFixed(1)}%</div>
                <div class="small muted">${s.winners}W / ${s.losers}L / ${s.breakevens}BE</div></div>
            <div class="card"><div class="label" data-i18n="view.r_distribution.card.avg_winner_loser">Avg winner / loser</div>
                <div class="value"><span class="pos">+${s.avg_winner_r.toFixed(2)}R</span> /
                    <span class="neg">${s.avg_loser_r.toFixed(2)}R</span></div>
                <div class="small muted">${esc(t('view.r_distribution.card.payoff', { ratio: s.payoff_ratio.toFixed(2) }))}</div></div>
            <div class="card"><div class="label" data-i18n="view.r_distribution.card.profit_factor">Profit factor</div>
                <div class="value ${s.profit_factor >= 1.5 ? 'pos' : s.profit_factor >= 1 ? '' : 'neg'}">${s.profit_factor.toFixed(2)}</div>
                <div class="small muted">${esc(t('view.r_distribution.card.max_win_loss', { maxWin: s.max_winner_r.toFixed(2), maxLoss: s.max_loser_r.toFixed(2) }))}</div></div>
            <div class="card"><div class="label" data-i18n="view.r_distribution.card.stdev_r">Stdev R</div>
                <div class="value">${s.stdev_r.toFixed(3)}</div></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.r_distribution.h2.histogram_0_5r_bins">Histogram (0.5R bins)</h2>
            ${histogramSvg(r.bins, s)}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.r_distribution.h2.per_tag_breakdown_sorted_by_sqn">Per-tag breakdown (sorted by SQN)</h2>
            ${tagTable(r.by_tag)}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.r_distribution.h2.tag_chart">Mean R + SQN per tag (top 20)</h2>
            <div id="rd-chart" style="width:100%;height:240px"></div>
        </div>
    `;
    try { applyUiI18n(el); } catch (_) {}
    renderTagChart(r.by_tag);
}

function renderTagChart(byTag) {
    const el = document.getElementById('rd-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const tags = (byTag || []).slice(0, 20);
    if (tags.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.r_distribution.empty_chart">${esc(t('view.r_distribution.empty_chart'))}</div>`;
        return;
    }
    const labels = tags.map(g => g.tag_name);
    const meanR = tags.map(g => Number(g.mean_r));
    const sqn = tags.map(g => Number(g.sqn));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.r_distribution.chart.tag_idx') },
            { label: t('view.r_distribution.chart.mean_r'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.r_distribution.chart.sqn'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 6, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.r_distribution.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, meanR, sqn, zero], el);
}

function histogramSvg(bins, stats) {
    const W = 1000, H = 260, padL = 40, padR = 10, padT = 10, padB = 36;
    const innerW = W - padL - padR, innerH = H - padT - padB;
    const maxCount = Math.max(...bins.map(b => b.count), 1);
    const barW = innerW / bins.length;
    const sy = (c) => padT + (1 - c / maxCount) * innerH;
    // Zero R line position (where bin starts at 0.0).
    const zeroIdx = bins.findIndex(b => b.low >= 0 - 1e-9);
    const zeroX = padL + zeroIdx * barW;
    const meanIdx = (stats.mean_r - bins[0].low) / 0.5;
    const meanX = padL + Math.max(0, Math.min(bins.length, meanIdx)) * barW;

    const rects = bins.map((b, i) => {
        const x = padL + i * barW;
        const y = sy(b.count);
        const h = padT + innerH - y;
        const color = b.low < 0 ? '#ff1f7a' : b.low >= 0 && b.high <= 0.5 ? '#9aa0c8' : '#7af0a8';
        return `<rect x="${x + 1}" y="${y}" width="${Math.max(1, barW - 2)}" height="${h}"
                fill="${color}" opacity="0.85"/>
                ${b.count > 0 ? `<text x="${x + barW / 2}" y="${y - 2}" text-anchor="middle"
                    fill="#cfd2e8" font-size="9">${b.count}</text>` : ''}`;
    }).join('');
    const labels = bins.map((b, i) => {
        const x = padL + i * barW + barW / 2;
        // Only label every other bin to avoid crowding.
        if (i % 2 !== 0) return '';
        return `<text x="${x}" y="${H - padB + 12}" text-anchor="middle"
                fill="#9aa0c8" font-size="9" transform="rotate(-35 ${x} ${H - padB + 12})">${b.low.toFixed(1)}</text>`;
    }).join('');
    return `<svg viewBox="0 0 ${W} ${H}" width="100%" style="display:block;">
        <rect x="${padL}" y="${padT}" width="${innerW}" height="${innerH}" fill="#0d0d22" stroke="#222"/>
        ${rects}
        <line x1="${zeroX}" y1="${padT}" x2="${zeroX}" y2="${padT + innerH}" stroke="#666" stroke-dasharray="3,3"/>
        <text x="${zeroX + 4}" y="${padT + 10}" fill="#9aa0c8" font-size="9">0R</text>
        <line x1="${meanX}" y1="${padT}" x2="${meanX}" y2="${padT + innerH}" stroke="#ffd24a" stroke-width="2"/>
        <text x="${meanX + 4}" y="${padT + 22}" fill="#ffd24a" font-size="10">${esc(t('view.r_distribution.svg.mean', { mean: stats.mean_r.toFixed(2) }))}</text>
        ${labels}
        <text x="${padL + innerW / 2}" y="${H - 4}" text-anchor="middle" fill="#9aa0c8" font-size="11">${esc(t('view.r_distribution.svg.x_axis'))}</text>
    </svg>`;
}

function tagTable(tags) {
    if (!tags.length) return '<p data-i18n="view.r_distribution.hint.no_tagged_trades_with_risk_amount_set" class="muted small">No tagged trades with risk_amount set.</p>';
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.r_distribution.th.tag">Tag</th><th data-i18n="view.r_distribution.th.samples">Samples</th><th data-i18n="view.r_distribution.th.mean_r">Mean R</th><th data-i18n="view.r_distribution.th.sqn">SQN</th>
        </tr></thead>
        <tbody>
        ${tags.map(t => {
            const sqnCls = t.sqn >= 2.5 ? 'pos' : t.sqn >= 1.6 ? '' : 'neg';
            return `<tr>
                <td><span style="display:inline-block;width:8px;height:8px;background:${esc(t.tag_color)};border-radius:50%;margin-right:6px;"></span>${esc(t.tag_name)}</td>
                <td>${t.samples}</td>
                <td class="${t.mean_r >= 0 ? 'pos' : 'neg'}">${(t.mean_r >= 0 ? '+' : '') + t.mean_r.toFixed(3)}R</td>
                <td class="${sqnCls}">${t.sqn.toFixed(2)}</td>
            </tr>`;
        }).join('')}
        </tbody></table>`;
}
