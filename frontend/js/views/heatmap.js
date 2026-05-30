// Finviz-style S&P 500 heatmap — color-coded grid by sector.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderHeatmap(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.heatmap.h1.market_heatmap" class="view-title">// MARKET HEATMAP</h1>
        <p data-i18n="view.heatmap.hint.150_s_p_500_names_grouped_by_gics_sector_colored_b" class="muted small">~150 S&P 500 names grouped by GICS sector, colored by today's % change. Your watchlist symbols add to a "Watchlist" pseudo-sector.</p>
        <div id="hm" data-i18n="common.loading">loading…</div>
    `;
    try {
        const r = await api.heatmap();
        if (!viewIsCurrent(tok)) return;
        renderTiles(r, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#hm');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderTiles(r, mount) {
    const bySector = new Map();
    for (const t of r.tiles) {
        const arr = bySector.get(t.sector) || [];
        arr.push(t);
        bySector.set(t.sector, arr);
    }
    // Sort sectors by total weight (count for now).
    const sectors = Array.from(bySector.entries())
        .sort((a, b) => b[1].length - a[1].length);

    const html = sectors.map(([sector, tiles]) => {
        const avg = tiles.reduce((a, t) => a + Number(t.change_pct), 0) / tiles.length;
        return `<div class="hm-sector">
            <div class="hm-sector-head">
                <span class="hm-sector-name">${esc(sector)}</span>
                <span class="${avg >= 0 ? 'pos' : 'neg'} small">${avg >= 0 ? '+' : ''}${avg.toFixed(2)}%</span>
            </div>
            <div class="hm-grid">
                ${tiles.sort((a, b) => Number(b.change_pct) - Number(a.change_pct)).map(t => {
                    const pct = Number(t.change_pct);
                    const intensity = Math.min(1, Math.abs(pct) / 4);
                    const color = pct >= 0
                        ? `rgba(35, 209, 96, ${0.15 + intensity * 0.7})`
                        : `rgba(255, 56, 96, ${0.15 + intensity * 0.7})`;
                    return `<a class="hm-tile" href="#research/${encodeURIComponent(t.symbol)}"
                        style="background:${color}" title="${esc(t.symbol)} · ${fmt(t.price)} · ${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%">
                        <span class="hm-sym">${esc(t.symbol)}</span>
                        <span class="hm-pct">${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%</span>
                    </a>`;
                }).join('')}
            </div>
        </div>`;
    }).join('');

    const el = mount.querySelector('#hm');
    if (el) el.innerHTML = html || '<p data-i18n="view.heatmap.hint.no_quotes_cached_yet_refresh_in_a_minute" class="muted">No quotes cached yet — refresh in a minute.</p>';
}
