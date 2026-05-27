// Short Interest tracker — Yahoo stats + FINRA daily short-volume.
import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc } from '../util.js';

const compact = (n) => {
    if (n == null) return '—';
    const abs = Math.abs(n);
    if (abs >= 1e9) return (n/1e9).toFixed(2)+'B';
    if (abs >= 1e6) return (n/1e6).toFixed(2)+'M';
    if (abs >= 1e3) return (n/1e3).toFixed(1)+'K';
    return n.toLocaleString();
};
const pct1 = (n) => n == null ? '—' : (n * 100).toFixed(2) + '%';
const pctSigned = (n) => n == null ? '—' : (n >= 0 ? '+' : '') + n.toFixed(2) + '%';

export async function renderShortInterest(mount, _state, sym) {
    if (sym) return renderSymbol(mount, sym.toUpperCase());
    const lists = await api.watchlists();
    mount.innerHTML = `
        <h1 class="view-title">// SHORT INTEREST</h1>
        <p class="muted small">Yahoo defaultKeyStatistics for shares short / float % / days-to-cover,
            FINRA Reg SHO daily short-volume aggregated across market centers.</p>

        <form id="sf" class="inline-form">
            <input name="sym" placeholder="symbol (GME)" style="text-transform:uppercase">
            <button class="primary" type="submit">Lookup</button>
        </form>

        <div class="chart-panel">
            <h2>Watchlist ranking (sorted by short % of float)</h2>
            <form id="rf" class="inline-form">
                <label>Universe
                    <select name="wl">
                        <option value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <button class="primary" type="submit">Rank</button>
            </form>
            <div id="ranked"></div>
        </div>
    `;
    document.getElementById('sf').addEventListener('submit', (e) => {
        e.preventDefault();
        const s = new FormData(e.target).get('sym').trim().toUpperCase();
        if (s) window.location.hash = `short-interest/${encodeURIComponent(s)}`;
    });
    document.getElementById('rf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const wid = fd.get('wl') || null;
        const el = document.getElementById('ranked');
        el.innerHTML = '<div class="boot">fetching…</div>';
        try {
            const rows = await api.shortRanked(wid);
            renderRanked(el, rows);
        } catch (err) {
            el.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderRanked(el, rows) {
    if (!rows.length) { el.innerHTML = '<p class="muted">No symbols in this universe.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr><th>#</th><th>Sym</th><th>Shares short</th><th>Prior month</th><th>Δ</th>
            <th>% Float</th><th>% Outstanding</th><th>Days to cover</th><th>Float</th></tr></thead>
        <tbody>${rows.map((r, i) => `
            <tr>
                <td>${i+1}</td>
                <td><a href="#short-interest/${encodeURIComponent(r.symbol)}">${esc(r.symbol)}</a></td>
                <td>${compact(r.shares_short)}</td>
                <td>${compact(r.shares_short_prior)}</td>
                <td class="${(r.change_pct ?? 0) >= 0 ? 'neg' : 'pos'}">${pctSigned(r.change_pct)}</td>
                <td>${pct1(r.short_pct_float)}</td>
                <td>${pct1(r.short_pct_outstanding)}</td>
                <td>${r.short_ratio != null ? r.short_ratio.toFixed(2) : '—'}</td>
                <td>${compact(r.float)}</td>
            </tr>`).join('')}</tbody></table>`;
}

async function renderSymbol(mount, sym) {
    mount.innerHTML = `
        <h1 class="view-title">// SHORT INTEREST · ${esc(sym)}
            <a class="link small" href="#short-interest">← back</a>
        </h1>
        <div id="ss-cards" class="cards">loading…</div>
        <div class="chart-panel">
            <h2>FINRA Reg SHO daily short volume (last 30 sessions)</h2>
            <div id="finra-vol"></div>
        </div>
        <div class="chart-panel">
            <h2>FINRA short % of total volume</h2>
            <div id="finra-pct"></div>
        </div>
    `;
    try {
        const [s, days] = await Promise.all([
            api.shortSymbol(sym),
            api.shortFinra(sym, 30).catch(() => []),
        ]);
        const changeCls = (s.change_pct ?? 0) >= 0 ? 'neg' : 'pos';
        document.getElementById('ss-cards').innerHTML = `
            <div class="card"><div class="label">Shares short</div><div class="value">${compact(s.shares_short)}</div></div>
            <div class="card"><div class="label">Prior month</div><div class="value">${compact(s.shares_short_prior)}</div></div>
            <div class="card"><div class="label">Δ vs prior</div>
                <div class="value ${changeCls}">${pctSigned(s.change_pct)}</div></div>
            <div class="card"><div class="label">% of float</div><div class="value">${pct1(s.short_pct_float)}</div></div>
            <div class="card"><div class="label">% of shares out</div><div class="value">${pct1(s.short_pct_outstanding)}</div></div>
            <div class="card"><div class="label">Days to cover</div><div class="value">${s.short_ratio != null ? s.short_ratio.toFixed(2) : '—'}</div></div>
            <div class="card"><div class="label">Float</div><div class="value">${compact(s.float)}</div></div>
        `;
        const labels = days.map(d => d.date);
        const vols   = days.map(d => Number(d.short_volume));
        const pcts   = days.map(d => Number(d.short_pct));
        if (days.length) {
            barChart(document.getElementById('finra-vol'), labels, vols, { color: '#ff2a6d' });
            barChart(document.getElementById('finra-pct'), labels, pcts, { color: '#b86bff' });
        } else {
            document.getElementById('finra-vol').innerHTML = '<p class="muted">No FINRA data — file may be embargoed or pending.</p>';
        }
    } catch (e) {
        document.getElementById('ss-cards').innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
