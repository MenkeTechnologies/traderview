// Dark Pool / Off-Exchange volume — FINRA TRF prints vs Yahoo total.
import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const pct = (n) => n == null ? '—' : (n * 100).toFixed(2) + '%';
const compact = (n) => {
    if (n == null) return '—';
    const abs = Math.abs(n);
    if (abs >= 1e9) return (n/1e9).toFixed(2)+'B';
    if (abs >= 1e6) return (n/1e6).toFixed(2)+'M';
    return n.toLocaleString();
};

export async function renderDarkpool(mount, _state, sym) {
    const tok = currentViewToken();
    if (sym) return renderSymbol(mount, sym.toUpperCase());
    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title">// DARK POOL / OFF-EXCHANGE VOLUME</h1>
        <p class="muted small">FINRA TRF off-exchange share volume / Yahoo consolidated total volume —
            a conservative proxy for ATS + internalizer / dark-pool printing.</p>

        <form id="df" class="inline-form">
            <input name="sym" placeholder="symbol (NVDA)" style="text-transform:uppercase">
            <button class="primary" type="submit">Lookup</button>
        </form>

        <div class="chart-panel">
            <h2>Watchlist ranking (avg off-exchange %)</h2>
            <form id="rf" class="inline-form">
                <label>Universe
                    <select name="wl">
                        <option value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <label>Days <input name="days" type="number" value="30" style="width:80px"></label>
                <button class="primary" type="submit">Rank</button>
            </form>
            <div id="dp-ranked"></div>
        </div>
    `;
    mount.querySelector('#df').addEventListener('submit', (e) => {
        e.preventDefault();
        const s = new FormData(e.target).get('sym').trim().toUpperCase();
        if (s) window.location.hash = `darkpool/${encodeURIComponent(s)}`;
    });
    mount.querySelector('#rf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const wid = fd.get('wl') || null;
        const days = Number(fd.get('days') || 30);
        const el = mount.querySelector('#dp-ranked');
        if (!el) return;
        el.innerHTML = '<div class="boot">scanning…</div>';
        try {
            const rows = await api.darkpoolRanked(wid, days);
            if (!viewIsCurrent(tok)) return;
            const el2 = mount.querySelector('#dp-ranked');
            if (el2) renderRanked(el2, rows);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const el2 = mount.querySelector('#dp-ranked');
            if (el2) el2.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderRanked(el, rows) {
    if (!rows.length) { el.innerHTML = '<p class="muted">No data in this universe yet.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr><th>#</th><th>Sym</th><th>Avg off-exch %</th><th>Latest</th><th>Sessions</th></tr></thead>
        <tbody>${rows.map((r, i) => `
            <tr>
                <td>${i+1}</td>
                <td><a href="#darkpool/${encodeURIComponent(r.symbol)}">${esc(r.symbol)}</a></td>
                <td>${pct(r.avg_off_exchange_pct)}</td>
                <td>${pct(r.latest_pct)}</td>
                <td>${r.samples}</td>
            </tr>`).join('')}</tbody></table>`;
}

async function renderSymbol(mount, sym) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// DARK POOL · ${esc(sym)}
            <a class="link small" href="#darkpool">← back</a>
        </h1>
        <div id="dp-cards" class="cards">loading…</div>
        <div class="chart-panel"><h2>Off-exchange share %</h2><div id="dp-pct"></div></div>
        <div class="chart-panel"><h2>Off-exchange volume (shares)</h2><div id="dp-vol"></div></div>
        <div class="chart-panel"><h2>Daily breakdown</h2><div id="dp-table"></div></div>
    `;
    try {
        const s = await api.darkpoolSymbol(sym, 60);
        if (!viewIsCurrent(tok)) return;
        const cardsEl = mount.querySelector('#dp-cards');
        if (!cardsEl) return;
        if (!s.days.length) {
            cardsEl.innerHTML =
                '<p class="muted">No overlap between FINRA TRF and Yahoo bars yet — fetch price bars first.</p>';
            return;
        }
        const latest = s.days[s.days.length - 1];
        cardsEl.innerHTML = `
            <div class="card"><div class="label">Avg off-exch %</div>
                <div class="value">${pct(s.avg_off_exchange_pct)}</div></div>
            <div class="card"><div class="label">Latest (${latest.date})</div>
                <div class="value">${pct(latest.off_exchange_pct)}</div></div>
            <div class="card"><div class="label">Latest off-exch vol</div>
                <div class="value">${compact(latest.off_exchange_volume)}</div></div>
            <div class="card"><div class="label">Latest total vol</div>
                <div class="value">${compact(latest.total_volume)}</div></div>
            <div class="card"><div class="label">Sessions</div>
                <div class="value">${s.days.length}</div></div>
        `;
        const labels = s.days.map(d => d.date);
        const pctEl = mount.querySelector('#dp-pct');
        const volEl = mount.querySelector('#dp-vol');
        if (pctEl) barChart(pctEl, labels, s.days.map(d => d.off_exchange_pct * 100), { color: '#b86bff' });
        if (volEl) barChart(volEl, labels, s.days.map(d => d.off_exchange_volume), { color: '#00e5ff' });
        const tableEl = mount.querySelector('#dp-table');
        if (tableEl) tableEl.innerHTML = `
            <table class="trades">
                <thead><tr><th>Date</th><th>Off-exchange vol</th><th>Total vol</th><th>Off %</th></tr></thead>
                <tbody>${s.days.slice().reverse().map(d => `
                    <tr>
                        <td>${esc(d.date)}</td>
                        <td>${compact(d.off_exchange_volume)}</td>
                        <td>${compact(d.total_volume)}</td>
                        <td>${pct(d.off_exchange_pct)}</td>
                    </tr>`).join('')}</tbody>
            </table>`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const cardsEl = mount.querySelector('#dp-cards');
        if (cardsEl) cardsEl.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
    void fmt;
}
