// Relative Rotation Graph — 11 sector ETFs vs SPY on (RS-Ratio,
// RS-Momentum) axes with comet tails. Canvas-drawn (no chart lib —
// RRG isn't a time series).

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const QUAD_COLOR = {
    leading: '#39ff14',
    weakening: '#ffd84a',
    lagging: '#ff3860',
    improving: '#00e5ff',
};

export async function renderRrg(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.rrg.h1" class="view-title">// RELATIVE ROTATION — sectors vs SPY</h1>
        <p class="muted small" data-i18n="view.rrg.subtitle">
            RS-Ratio (x) vs RS-Momentum (y). Clockwise rotation: improving → leading →
            weakening → lagging. Tails show the last ~8 weeks.
        </p>
        <div id="rrg-market" class="cards"></div>
        <div class="chart-panel">
            <canvas id="rrg-canvas" width="900" height="620"></canvas>
        </div>
        <div class="chart-panel"><div id="rrg-table">
            <span class="tv-spinner-inline" role="status" aria-label="loading"></span>
        </div></div>
    `;
    try { applyUiI18n(mount); } catch (_) {}

    // Market-level gauges ride alongside the rotation chart.
    Promise.all([
        api.marketFedModel().catch(() => null),
        api.marketNhNl().catch(() => null),
    ]).then(([fed, nhnl]) => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rrg-market');
        if (!el) return;
        const fedCard = fed ? `
            <div class="card"><div class="label">Fed model spread</div>
                <div class="value ${fed.spread_pct > 0 ? 'pos' : 'neg'}">${(fed.spread_pct >= 0 ? '+' : '') + fed.spread_pct.toFixed(2)}pp</div>
                <div class="small muted">earnings yield ${fed.earnings_yield_pct.toFixed(2)}% vs 10Y ${fed.treasury_10y_pct.toFixed(2)}% — ${esc(fed.verdict.replace('_', ' '))}</div>
            </div>` : '';
        const nhnlCard = nhnl ? `
            <div class="card"><div class="label">52-wk NH − NL</div>
                <div class="value ${nhnl.nh_nl_diff > 0 ? 'pos' : nhnl.nh_nl_diff < 0 ? 'neg' : ''}">${nhnl.nh_nl_diff > 0 ? '+' : ''}${nhnl.nh_nl_diff}</div>
                <div class="small muted">${nhnl.new_highs.length} highs / ${nhnl.new_lows.length} lows of ${nhnl.evaluated} names (${nhnl.nh_nl_pct.toFixed(0)}%)</div>
            </div>` : '';
        el.innerHTML = fedCard + nhnlCard;
    });

    const report = await api.rrg().catch(() => null);
    if (!viewIsCurrent(tok)) return;
    const tableEl = mount.querySelector('#rrg-table');
    if (!report || !report.entries.length) {
        tableEl.innerHTML = `<div class="boot muted">No RRG data${report?.errors?.length ? ' — ' + esc(report.errors.join('; ')) : ''}.</div>`;
        return;
    }
    drawRrg(mount.querySelector('#rrg-canvas'), report.entries);
    tableEl.innerHTML = `
        <table class="gs-table">
            <thead><tr><th>Sector</th><th>Quadrant</th><th>RS-Ratio</th><th>RS-Momentum</th></tr></thead>
            <tbody>
                ${report.entries.map(e => `
                    <tr>
                        <td><a href="#research/${encodeURIComponent(e.ticker)}">${esc(e.ticker)}</a> <span class="muted small">${esc(e.name)}</span></td>
                        <td><span style="color:${QUAD_COLOR[e.quadrant] || '#aab'}">${esc(e.quadrant.toUpperCase())}</span></td>
                        <td>${e.current.rs_ratio.toFixed(2)}</td>
                        <td>${e.current.rs_momentum.toFixed(2)}</td>
                    </tr>`).join('')}
            </tbody>
        </table>`;
}

function drawRrg(canvas, entries) {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    ctx.clearRect(0, 0, W, H);

    // Scale: center 100/100; span fits all points + tails with padding.
    let span = 2.0;
    for (const e of entries) {
        for (const p of e.tail) {
            span = Math.max(span, Math.abs(p.rs_ratio - 100), Math.abs(p.rs_momentum - 100));
        }
    }
    span *= 1.2;
    const x = (v) => W / 2 + ((v - 100) / span) * (W / 2 - 30);
    const y = (v) => H / 2 - ((v - 100) / span) * (H / 2 - 30);

    // Quadrant tints.
    ctx.globalAlpha = 0.07;
    ctx.fillStyle = QUAD_COLOR.leading;
    ctx.fillRect(W / 2, 0, W / 2, H / 2);
    ctx.fillStyle = QUAD_COLOR.weakening;
    ctx.fillRect(W / 2, H / 2, W / 2, H / 2);
    ctx.fillStyle = QUAD_COLOR.lagging;
    ctx.fillRect(0, H / 2, W / 2, H / 2);
    ctx.fillStyle = QUAD_COLOR.improving;
    ctx.fillRect(0, 0, W / 2, H / 2);
    ctx.globalAlpha = 1;

    // Axes through 100/100.
    ctx.strokeStyle = 'rgba(170,170,187,0.4)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, H / 2); ctx.lineTo(W, H / 2);
    ctx.moveTo(W / 2, 0); ctx.lineTo(W / 2, H);
    ctx.stroke();

    // Quadrant labels.
    ctx.font = '11px "Share Tech Mono", monospace';
    ctx.fillStyle = 'rgba(170,170,187,0.6)';
    ctx.fillText('IMPROVING', 12, 18);
    ctx.fillText('LEADING', W - 80, 18);
    ctx.fillText('LAGGING', 12, H - 10);
    ctx.fillText('WEAKENING', W - 92, H - 10);

    for (const e of entries) {
        const color = QUAD_COLOR[e.quadrant] || '#aab';
        // Tail.
        ctx.strokeStyle = color;
        ctx.globalAlpha = 0.5;
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        e.tail.forEach((p, i) => {
            const px = x(p.rs_ratio), py = y(p.rs_momentum);
            if (i === 0) ctx.moveTo(px, py); else ctx.lineTo(px, py);
        });
        ctx.stroke();
        ctx.globalAlpha = 1;
        // Head dot + label.
        const hx = x(e.current.rs_ratio), hy = y(e.current.rs_momentum);
        ctx.fillStyle = color;
        ctx.beginPath();
        ctx.arc(hx, hy, 5, 0, Math.PI * 2);
        ctx.fill();
        ctx.font = 'bold 12px "Share Tech Mono", monospace';
        ctx.fillText(e.ticker, hx + 8, hy + 4);
    }
}
