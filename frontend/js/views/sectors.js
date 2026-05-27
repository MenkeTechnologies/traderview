import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderSectors(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// SECTOR STRENGTH</h1>
        <p class="muted small">11 SPDR sector ETFs ranked by today's % change and relative strength vs SPY (ZenBot-style).</p>
        <div id="sec"><div class="boot">loading…</div></div>
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
                    <thead><tr><th>#</th><th>Sector</th><th>ETF</th><th>Price</th>
                        <th>Change %</th><th>RS vs SPY</th><th>Bar</th></tr></thead>
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
