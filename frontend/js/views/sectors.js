import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderSectors(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.sectors.h1.sector_strength" class="view-title">// SECTOR STRENGTH</h1>
        <p data-i18n="view.sectors.hint.11_spdr_sector_etfs_ranked_by_today_s_change_and_r" class="muted small">11 SPDR sector ETFs ranked by today's % change and relative strength vs SPY (ZenBot-style).</p>
        <div id="sec"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>
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
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const secEl = mount.querySelector('#sec');
        if (secEl) secEl.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
