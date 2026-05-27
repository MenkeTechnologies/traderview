// Stock scanners — Warrior/Zendoo presets across the user's watchlist universe.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PRESETS = [
    { id: 'premarket_gappers', label: 'Premarket Gappers',  desc: '≥ 5% gap (open vs prior close)' },
    { id: 'momentum_movers',   label: 'Momentum Movers',    desc: '≥ 5% move + 2× rel-volume' },
    { id: 'low_float_runners', label: 'Low-Float Runners',  desc: '≥ 10% move + 5× rel-volume' },
    { id: 'high_of_day',       label: 'High of Day',        desc: 'within 0.5% of session high' },
    { id: 'volume_surge',      label: 'Volume Surge',       desc: '≥ 3× 20-day avg volume' },
    { id: 'breakout',          label: 'Breakout',           desc: 'at session high + green day' },
    { id: 'breakdown',         label: 'Breakdown',          desc: '≤ −5% on the day' },
    { id: 'pct52w_high',       label: '52w Highs',          desc: 'within 1% of 52-week high' },
    { id: 'pct52w_low',        label: '52w Lows',           desc: 'within 1% of 52-week low' },
    { id: 'oversold_bounce',   label: 'Oversold Bounce',    desc: 'green day off oversold base' },
];

export async function renderScanners(mount) {
    const tok = currentViewToken();
    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title">// SCANNERS</h1>
        <p class="muted small">Warrior/Zendoo-style preset scans across your watchlist universe.
        Click a preset to run.</p>

        <div class="chart-panel">
            <label>Universe
                <select id="wl">
                    <option value="">all my watchlists</option>
                    ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                </select>
            </label>
        </div>

        <div class="scanner-grid">
            ${PRESETS.map(p => `
                <button class="scanner-tile" data-preset="${p.id}">
                    <div class="scanner-title">${esc(p.label)}</div>
                    <div class="scanner-desc">${esc(p.desc)}</div>
                </button>`).join('')}
        </div>

        <div id="scan-result"></div>
    `;
    mount.querySelectorAll('[data-preset]').forEach(b =>
        b.addEventListener('click', async () => {
            const wlEl = mount.querySelector('#wl');
            const wid = (wlEl && wlEl.value) || null;
            const el = mount.querySelector('#scan-result');
            if (!el) return;
            el.innerHTML = '<div class="boot">scanning…</div>';
            mount.querySelectorAll('.scanner-tile').forEach(t => t.classList.toggle('active', t === b));
            try {
                const r = await api.scanRun(b.dataset.preset, wid, 100);
                if (!viewIsCurrent(tok)) return;
                const elNow = mount.querySelector('#scan-result');
                if (elNow) elNow.innerHTML = renderHits(r);
            } catch (e) {
                if (!viewIsCurrent(tok)) return;
                const elNow = mount.querySelector('#scan-result');
                if (elNow) elNow.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
            }
        }));
}

function renderHits(r) {
    return `<div class="chart-panel">
        <h2>${esc(r.label)} · ${r.hits.length} hits of ${r.universe_size} scanned</h2>
        ${r.hits.length ? `<table class="trades">
            <thead><tr>
                <th>Symbol</th><th>Price</th><th>Gap%</th><th>Day%</th><th>Δ vs prior</th>
                <th>Vol</th><th>RVol</th><th>HOD dist</th><th>52w</th>
            </tr></thead><tbody>${r.hits.map(h => {
                const cls = h.change_pct >= 0 ? 'pos' : 'neg';
                return `<tr>
                    <td><a href="#research/${encodeURIComponent(h.symbol)}">${esc(h.symbol)}</a></td>
                    <td>${fmt(h.price)}</td>
                    <td class="${h.gap_pct >= 0 ? 'pos' : 'neg'}">${fmt(h.gap_pct, 2)}%</td>
                    <td class="${h.day_pct >= 0 ? 'pos' : 'neg'}">${fmt(h.day_pct, 2)}%</td>
                    <td class="${cls}">${fmt(h.change_pct, 2)}%</td>
                    <td>${h.volume.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${fmt(h.rel_volume, 2)}×</td>
                    <td>${fmt(h.hod_dist_pct, 2)}%</td>
                    <td>${fmt(h.year_high_pct, 1)}% / ${fmt(h.year_low_pct, 1)}%</td>
                </tr>`;
            }).join('')}</tbody></table>` : '<p class="muted">No matches.</p>'}
    </div>`;
}
