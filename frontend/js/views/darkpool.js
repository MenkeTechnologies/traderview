// Dark Pool / Off-Exchange volume — FINRA TRF prints vs Yahoo total.
import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n, t } from '../i18n.js';

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
        <h1 data-i18n="view.darkpool.h1.dark_pool_off_exchange_volume" class="view-title">// DARK POOL / OFF-EXCHANGE VOLUME</h1>
        <p data-i18n="view.darkpool.hint.finra_trf_off_exchange_share_volume_yahoo_consolid" class="muted small">FINRA TRF off-exchange share volume / Yahoo consolidated total volume —
            a conservative proxy for ATS + internalizer / dark-pool printing.</p>

        <form id="df" class="inline-form">
            <input name="sym" placeholder="symbol (NVDA)" data-i18n-placeholder="view.darkpool.placeholder.symbol" style="text-transform:uppercase">
            <button data-i18n="view.darkpool.btn.lookup" class="primary" type="submit">Lookup</button>
        </form>

        <div class="chart-panel">
            <h2 data-i18n="view.darkpool.h2.watchlist_ranking_avg_off_exchange">Watchlist ranking (avg off-exchange %)</h2>
            <form id="rf" class="inline-form">
                <label><span data-i18n="view.darkpool.label.universe">Universe</span>
                    <select name="wl">
                        <option data-i18n="view.darkpool.opt.all_my_watchlists" value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.darkpool.label.days">Days</span>
                    <input name="days" type="number" value="30" style="width:80px"></label>
                <button data-i18n="view.darkpool.btn.rank" class="primary" type="submit">Rank</button>
            </form>
            <div id="dp-ranked"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.darkpool.h2.ranked_chart">Avg off-exchange % by symbol</h2>
            <div id="dp-ranked-chart" style="width:100%;height:240px"></div>
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
        el.innerHTML = '<div class="boot" data-i18n="common.status.scanning">scanning…</div>';
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
    if (!rows.length) { el.innerHTML = '<p data-i18n="view.darkpool.hint.no_data_in_this_universe_yet" class="muted">No data in this universe yet.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr><th>#</th><th data-i18n="view.darkpool.th.sym">Sym</th><th data-i18n="view.darkpool.th.avg_off_exch">Avg off-exch %</th><th data-i18n="view.darkpool.th.latest">Latest</th><th data-i18n="view.darkpool.th.sessions">Sessions</th></tr></thead>
        <tbody>${rows.map((r, i) => `
            <tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td>${i+1}</td>
                <td><a href="#darkpool/${encodeURIComponent(r.symbol)}">${esc(r.symbol)}</a></td>
                <td>${pct(r.avg_off_exchange_pct)}</td>
                <td>${pct(r.latest_pct)}</td>
                <td>${r.samples}</td>
            </tr>`).join('')}</tbody></table>`;
    renderRankedChart(rows);
}

function renderRankedChart(rows) {
    const el = document.getElementById('dp-ranked-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const top = (rows || []).slice(0, 30).filter(r => Number.isFinite(Number(r.avg_off_exchange_pct)));
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.darkpool.empty_chart">${esc(t('view.darkpool.empty_chart'))}</div>`;
        return;
    }
    const labels = top.map(r => r.symbol);
    const ys = top.map(r => Number(r.avg_off_exchange_pct) * 100);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.darkpool.chart.sym_idx') },
            { label: t('view.darkpool.chart.avg_pct'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 10, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

async function renderSymbol(mount, sym) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// DARK POOL · ${esc(sym)}
            <a class="link small" href="#darkpool">← back</a>
        </h1>
        <div id="dp-cards" class="cards" data-i18n="common.loading">loading…</div>
        <div class="chart-panel"><h2 data-i18n="view.darkpool.h2.off_exchange_share">Off-exchange share %</h2><div id="dp-pct"></div></div>
        <div class="chart-panel"><h2 data-i18n="view.darkpool.h2.off_exchange_volume_shares">Off-exchange volume (shares)</h2><div id="dp-vol"></div></div>
        <div class="chart-panel"><h2 data-i18n="view.darkpool.h2.daily_breakdown">Daily breakdown</h2><div id="dp-table"></div></div>
    `;
    try {
        const s = await api.darkpoolSymbol(sym, 60);
        if (!viewIsCurrent(tok)) return;
        const cardsEl = mount.querySelector('#dp-cards');
        if (!cardsEl) return;
        if (!s.days.length) {
            cardsEl.innerHTML =
                '<p data-i18n="view.darkpool.hint.no_overlap_between_finra_trf_and_yahoo_bars_yet_fe" class="muted">No overlap between FINRA TRF and Yahoo bars yet — fetch price bars first.</p>';
            return;
        }
        const latest = s.days[s.days.length - 1];
        cardsEl.innerHTML = `
            <div class="card"><div class="label" data-i18n="view.darkpool.card.avg_off_exch_pct">Avg off-exch %</div>
                <div class="value">${pct(s.avg_off_exchange_pct)}</div></div>
            <div class="card"><div class="label"><span data-i18n="view.darkpool.card.latest">Latest</span> (${latest.date})</div>
                <div class="value">${pct(latest.off_exchange_pct)}</div></div>
            <div class="card"><div class="label" data-i18n="view.darkpool.card.latest_off_exch_vol">Latest off-exch vol</div>
                <div class="value">${compact(latest.off_exchange_volume)}</div></div>
            <div class="card"><div class="label" data-i18n="view.darkpool.card.latest_total_vol">Latest total vol</div>
                <div class="value">${compact(latest.total_volume)}</div></div>
            <div class="card"><div class="label" data-i18n="view.darkpool.card.sessions">Sessions</div>
                <div class="value">${s.days.length}</div></div>
        `;
        try { applyUiI18n(cardsEl); } catch (_) {}
        const labels = s.days.map(d => d.date);
        const pctEl = mount.querySelector('#dp-pct');
        const volEl = mount.querySelector('#dp-vol');
        if (pctEl) barChart(pctEl, labels, s.days.map(d => d.off_exchange_pct * 100), { color: '#b86bff' });
        if (volEl) barChart(volEl, labels, s.days.map(d => d.off_exchange_volume), { color: '#00e5ff' });
        const tableEl = mount.querySelector('#dp-table');
        if (tableEl) tableEl.innerHTML = `
            <table class="trades">
                <thead><tr><th data-i18n="view.darkpool.th.date">Date</th><th data-i18n="view.darkpool.th.off_exchange_vol">Off-exchange vol</th><th data-i18n="view.darkpool.th.total_vol">Total vol</th><th data-i18n="view.darkpool.th.off">Off %</th></tr></thead>
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
