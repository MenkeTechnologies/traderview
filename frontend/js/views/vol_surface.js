// Implied-volatility surface — heatmap matrix + ATM term structure + skew curve.
import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderVolSurface(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.vol_surface.h1.vol_surface" class="view-title">// VOL SURFACE</h1>
        <p class="muted small" data-i18n="view.vol_surface.hint.intro">IV grid across moneyness × expiration. Color intensity shows IV relative to the surface min/max — bright red = highest IV (richest options), deep blue = lowest (cheapest). Use OTM puts (negative moneyness) and OTM calls (positive) to read skew. Watch for term-structure inversion (front-month IV > back-month): usually signals an upcoming binary event.</p>

        <form id="vsForm" class="filter-form">
            <label><span data-i18n="view.vol_surface.label.symbol">Symbol</span>
                <input type="text" id="vsSym" value="SPY" required></label>
            <label><span data-i18n="view.vol_surface.label.n_expirations"># expirations</span>
                <input type="number" id="vsN" value="8" min="1" max="16"></label>
            <button data-i18n="view.vol_surface.btn.build_surface" type="submit" class="btn">Build surface</button>
        </form>

        <div id="vsOut"><p data-i18n="view.vol_surface.hint.enter_a_symbol_to_fetch_its_surface" class="muted small">Enter a symbol to fetch its surface.</p></div>
    `;
    mount.querySelector('#vsForm').addEventListener('submit', async (e) => {
        e.preventDefault();
        const symEl = mount.querySelector('#vsSym');
        const nEl = mount.querySelector('#vsN');
        const sym = symEl ? symEl.value.trim().toUpperCase() : '';
        const n = (nEl && parseInt(nEl.value, 10)) || 8;
        await fetchAndRender(sym, n, mount, tok);
    });
}

async function fetchAndRender(sym, n, mount, tok) {
    const out = mount.querySelector('#vsOut');
    if (!out) return;
    out.innerHTML = `<p class="muted small">${esc(t('view.vol_surface.hint.fetching', { sym, n }))}</p>`;
    try {
        const s = await api.volSurface(sym, n);
        if (!viewIsCurrent(tok)) return;
        const outNow = mount.querySelector('#vsOut');
        if (outNow) renderSurface(s, outNow, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const outNow = mount.querySelector('#vsOut');
        if (outNow) outNow.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderSurface(s, out, mount) {
    if (!s.expirations.length) {
        out.innerHTML = `<p class="boot">${esc(t('view.vol_surface.empty', { sym: s.symbol }))}</p>`;
        return;
    }
    out.innerHTML = `
        <div class="chart-panel">
            <h2>${esc(t('view.vol_surface.h2.symbol_spot', { symbol: s.symbol, spot: s.spot.toFixed(2) }))}</h2>
            <p class="muted small">${esc(t('view.vol_surface.stats', {
                exps: s.expirations.length,
                buckets: s.moneyness.length,
                time: new Date(s.fetched_at).toLocaleTimeString(undefined, { hour12: false }),
            }))}</p>
            <div id="vsHeat"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.vol_surface.h2.atm_term_structure">ATM term structure</h2>
            <div id="vsTerm"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.vol_surface.h2.front_month_skew">Front-month skew</h2>
            <div id="vsSkew"></div>
        </div>
    `;
    renderHeatmap(s, mount);
    renderTermSvg(s, mount);
    renderSkewSvg(s, mount);
}

function renderHeatmap(s, mount) {
    const all = s.expirations.flatMap(r => r.iv_by_moneyness).filter(v => v != null);
    const min = Math.min(...all);
    const max = Math.max(...all);
    const ivCol = (iv) => {
        if (iv == null) return 'transparent';
        const t = (iv - min) / Math.max(max - min, 1e-9);
        // blue (low) -> green -> yellow -> red (high)
        const r = Math.round(t * 255);
        const b = Math.round((1 - t) * 255);
        const g = Math.round(Math.max(0, 255 - Math.abs(t - 0.5) * 510));
        return `rgb(${r},${g},${b})`;
    };
    const moneynessLabel = (m) =>
        m === 0 ? 'ATM' : `${m > 0 ? '+' : ''}${(m * 100).toFixed(0)}%`;
    const rowHtml = (row) => `<tr>
        <td class="muted small">${row.expiration} <span class="muted">(${row.days_to_expiry}d)</span></td>
        ${row.iv_by_moneyness.map(iv => {
            const txt = iv == null ? '—' : (iv * 100).toFixed(1);
            return `<td style="background:${ivCol(iv)};color:#000;text-align:center;font-family:'Share Tech Mono',monospace;">${txt}</td>`;
        }).join('')}
        <td class="small">${row.atm_iv == null ? '—' : (row.atm_iv * 100).toFixed(1) + '%'}</td>
    </tr>`;
    const heatEl = mount.querySelector('#vsHeat');
    if (!heatEl) return;
    heatEl.innerHTML = `
        <table class="trades" style="table-layout:fixed;">
            <thead><tr>
                <th data-i18n="view.vol_surface.th.expiration">Expiration</th>
                ${s.moneyness.map(m => `<th>${moneynessLabel(m)}</th>`).join('')}
                <th data-i18n="view.vol_surface.th.atm_iv">ATM IV</th>
            </tr></thead>
            <tbody>${s.expirations.map(rowHtml).join('')}</tbody>
        </table>
        <p class="muted small">${esc(t('view.vol_surface.range', { min: (min * 100).toFixed(1), max: (max * 100).toFixed(1) }))}</p>
    `;
}

function renderTermSvg(s, mount) {
    const termEl = mount.querySelector('#vsTerm');
    if (!termEl) return;
    const pts = s.term_structure;
    if (!pts.length) { termEl.innerHTML = '<p data-i18n="view.vol_surface.hint.no_atm_iv_resolved" class="muted small">no ATM IV resolved</p>'; return; }
    const w = 700, h = 220, pad = 40;
    const xs = pts.map(p => p.days_to_expiry);
    const ys = pts.map(p => p.atm_iv * 100);
    const xMin = Math.min(...xs), xMax = Math.max(...xs);
    const yMin = Math.min(...ys), yMax = Math.max(...ys);
    const sx = (x) => pad + (x - xMin) / Math.max(xMax - xMin, 1) * (w - 2 * pad);
    const sy = (y) => h - pad - (y - yMin) / Math.max(yMax - yMin, 1e-9) * (h - 2 * pad);
    const path = pts.map((p, i) => (i ? 'L' : 'M') + sx(p.days_to_expiry) + ',' + sy(p.atm_iv * 100)).join(' ');
    const dots = pts.map(p => `<circle cx="${sx(p.days_to_expiry)}" cy="${sy(p.atm_iv * 100)}" r="3" fill="#00ffaa"/>`).join('');
    const front = ys[0], back = ys[ys.length - 1];
    const inverted = front > back;
    termEl.innerHTML = `
        <svg viewBox="0 0 ${w} ${h}" width="100%" style="display:block;">
            <line x1="${pad}" y1="${h - pad}" x2="${w - pad}" y2="${h - pad}" stroke="#444"/>
            <line x1="${pad}" y1="${pad}" x2="${pad}" y2="${h - pad}" stroke="#444"/>
            <path d="${path}" stroke="#00e5ff" stroke-width="2" fill="none"/>
            ${dots}
            <text x="${w / 2}" y="${h - 10}" text-anchor="middle" fill="#9aa0c8" font-size="11">${esc(t('view.vol_surface.axis.x'))}</text>
            <text x="12" y="${h / 2}" fill="#9aa0c8" font-size="11" transform="rotate(-90 12 ${h / 2})">${esc(t('view.vol_surface.axis.y'))}</text>
        </svg>
        <p class="small ${inverted ? 'neg' : ''}">
            ${esc(t('view.vol_surface.term_summary', {
                front:   front.toFixed(1),
                back:    back.toFixed(1),
                verdict: t(inverted ? 'view.vol_surface.inverted' : 'view.vol_surface.normal'),
            }))}
        </p>
    `;
}

function renderSkewSvg(s, mount) {
    const skewEl = mount.querySelector('#vsSkew');
    if (!skewEl) return;
    const pts = s.front_skew;
    if (!pts.length) { skewEl.innerHTML = '<p data-i18n="view.vol_surface.hint.no_skew_data" class="muted small">no skew data</p>'; return; }
    const w = 700, h = 220, pad = 40;
    const xs = pts.map(p => p.moneyness * 100);
    const allY = pts.flatMap(p => [p.call_iv, p.put_iv]).filter(v => v != null).map(v => v * 100);
    if (!allY.length) { skewEl.innerHTML = '<p data-i18n="view.vol_surface.hint.no_skew_ivs_resolved" class="muted small">no skew IVs resolved</p>'; return; }
    const xMin = Math.min(...xs), xMax = Math.max(...xs);
    const yMin = Math.min(...allY), yMax = Math.max(...allY);
    const sx = (x) => pad + (x - xMin) / Math.max(xMax - xMin, 1) * (w - 2 * pad);
    const sy = (y) => h - pad - (y - yMin) / Math.max(yMax - yMin, 1e-9) * (h - 2 * pad);
    const linePath = (key) => pts
        .filter(p => p[key] != null)
        .map((p, i) => (i ? 'L' : 'M') + sx(p.moneyness * 100) + ',' + sy(p[key] * 100))
        .join(' ');
    const dots = (key, col) => pts
        .filter(p => p[key] != null)
        .map(p => `<circle cx="${sx(p.moneyness * 100)}" cy="${sy(p[key] * 100)}" r="3" fill="${col}"/>`)
        .join('');
    skewEl.innerHTML = `
        <svg viewBox="0 0 ${w} ${h}" width="100%" style="display:block;">
            <line x1="${pad}" y1="${h - pad}" x2="${w - pad}" y2="${h - pad}" stroke="#444"/>
            <line x1="${pad}" y1="${pad}" x2="${pad}" y2="${h - pad}" stroke="#444"/>
            <line x1="${sx(0)}" y1="${pad}" x2="${sx(0)}" y2="${h - pad}" stroke="#666" stroke-dasharray="3,3"/>
            <path d="${linePath('call_iv')}" stroke="#00ffaa" stroke-width="2" fill="none"/>
            <path d="${linePath('put_iv')}"  stroke="#ff1f7a" stroke-width="2" fill="none"/>
            ${dots('call_iv', '#00ffaa')}
            ${dots('put_iv', '#ff1f7a')}
            <text x="${w / 2}" y="${h - 10}" text-anchor="middle" fill="#9aa0c8" font-size="11">${esc(t('view.vol_surface.svg.x_axis'))}</text>
            <text x="12" y="${h / 2}" fill="#9aa0c8" font-size="11" transform="rotate(-90 12 ${h / 2})">${esc(t('view.vol_surface.svg.y_axis'))}</text>
            <text x="${w - 90}" y="${pad + 12}" fill="#00ffaa" font-size="11">${esc(t('view.vol_surface.svg.calls'))}</text>
            <text x="${w - 90}" y="${pad + 28}" fill="#ff1f7a" font-size="11">${esc(t('view.vol_surface.svg.puts'))}</text>
        </svg>
    `;
}
