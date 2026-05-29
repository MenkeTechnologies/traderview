// Sector ETF rotation heatmap — 11 sectors × 3 windows ranked by RS vs SPY,
// plus a 60-day RS-vs-SPY sparkline per row.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderSectorRotation(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.sector_rotation.h1.sector_rotation" class="view-title">// SECTOR ROTATION</h1>
        <p class="muted small" data-i18n="view.sector_rotation.hint.intro">For each SPDR sector ETF (XLK/XLF/XLE/XLV/XLY/XLP/XLI/XLB/XLU/XLRE/XLC), relative-strength versus SPY across 5/20/60-day windows. Ranks color the cells: rank 1-3 = green (leadership), 4-7 = grey (in line), 8-11 = red (laggards). The sparkline column shows daily sector_return − SPY_return for the last 60 sessions — a rising line means the sector is gaining ground on the index. Refreshes every 5 min.</p>

        <div id="sr-out"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
    `;
    await refresh(mount, tok);
    const t = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(t); return; }
        refresh(mount, tok);
    }, 5 * 60_000);
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#sector-rotation')) clearInterval(t);
    }, { once: true });
}

async function refresh(mount, tok) {
    try {
        const r = await api.sectorRotation();
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#sr-out');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function rankColor(rank, n) {
    if (rank == null) return '#1a1d2e';
    // 1..3 → green, 4..7 → grey, 8..n → red
    if (rank <= 3) {
        const t = (4 - rank) / 3; // 1,0.66,0.33
        return `rgba(122,240,168,${0.35 + 0.5 * t})`;
    }
    if (rank >= 8) {
        const t = (rank - 7) / (n - 7);
        return `rgba(255,31,122,${0.30 + 0.5 * t})`;
    }
    return 'rgba(154,160,200,0.18)';
}

function sparkSvg(values, h = 28, w = 160) {
    if (!values.length) return '';
    const lo = Math.min(...values, 0);
    const hi = Math.max(...values, 0);
    const sx = (i) => (i / Math.max(values.length - 1, 1)) * w;
    const sy = (v) => h - ((v - lo) / Math.max(hi - lo, 1e-9)) * h;
    const last = values[values.length - 1];
    const color = last >= 0 ? '#7af0a8' : '#ff1f7a';
    const path = values.map((v, i) => (i ? 'L' : 'M') + sx(i).toFixed(1) + ',' + sy(v).toFixed(1)).join(' ');
    const zeroY = sy(0);
    return `<svg viewBox="0 0 ${w} ${h}" width="${w}" height="${h}" style="display:block;">
        <line x1="0" y1="${zeroY}" x2="${w}" y2="${zeroY}" stroke="#444" stroke-dasharray="2,3"/>
        <path d="${path}" stroke="${color}" stroke-width="1.5" fill="none"/>
    </svg>`;
}

function render(r, mount) {
    const n = r.sectors.length;
    const headers = r.windows.map(w => `<th>${esc(w)}</th>`).join('');
    const leadership = r.leadership_by_window.map((arr, i) =>
        `<div><strong>${esc(r.windows[i])}:</strong> ${arr.map(esc).join(' · ')}</div>`
    ).join('');
    const rows = r.sectors.map(s => {
        const cells = s.windows.map(w => {
            const bg = rankColor(w.rank, n);
            const ret = w.return_pct == null ? '—' : (w.return_pct >= 0 ? '+' : '') + w.return_pct.toFixed(2) + '%';
            const rs  = w.rs_pct == null ? '—' : (w.rs_pct >= 0 ? '+' : '') + w.rs_pct.toFixed(2) + ' RS';
            const rk  = w.rank == null ? '—' : `#${w.rank}`;
            return `<td style="background:${bg};color:#000;text-align:center;padding:4px;">
                <div style="font-weight:700;">${rk}</div>
                <div style="font-size:10px;">${ret}</div>
                <div style="font-size:10px;opacity:0.7;">${rs}</div>
            </td>`;
        }).join('');
        return `<tr>
            <td><strong>${esc(s.symbol)}</strong>
                <span class="muted small">${esc(s.label)}</span></td>
            ${cells}
            <td>${sparkSvg(s.rs_sparkline)}</td>
        </tr>`;
    }).join('');

    const out = mount.querySelector('#sr-out');
    if (!out) return;
    out.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.sector_rotation.h2.leadership">Leadership</h2>
            <div class="muted small">${leadership}</div>
            <p class="muted small" style="margin-top:6px;">
                SPY benchmark returns: ${r.windows.map((w, i) =>
                    `<strong>${esc(w)}</strong> ${(r.spy_returns[i] >= 0 ? '+' : '') + r.spy_returns[i].toFixed(2)}%`
                ).join(' · ')}
            </p>
        </div>
        <div class="chart-panel">
            <h2>${esc(t('view.sector_rotation.h2.heatmap', { sectors: n, windows: r.windows.length }))}</h2>
            <table class="corr-matrix">
                <thead><tr><th data-i18n="view.sector_rotation.th.sector">Sector</th>${headers}<th data-i18n="view.sector_rotation.th.60_day_rs_line">60-day RS line</th></tr></thead>
                <tbody>${rows}</tbody>
            </table>
            <p class="muted small">${esc(t('view.sector_rotation.hint.computed', { time: new Date(r.computed_at).toLocaleString() }))}</p>
        </div>
    `;
    void fmt;
}
