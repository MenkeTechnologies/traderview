// Short Interest tracker — Yahoo stats + FINRA daily short-volume.
import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n } from '../i18n.js';

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
    const tok = currentViewToken();
    if (sym) return renderSymbol(mount, sym.toUpperCase(), tok);
    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.short_interest.h1.short_interest" class="view-title">// SHORT INTEREST</h1>
        <p data-i18n="view.short_interest.hint.yahoo_defaultkeystatistics_for_shares_short_float_" class="muted small">Yahoo defaultKeyStatistics for shares short / float % / days-to-cover,
            FINRA Reg SHO daily short-volume aggregated across market centers.</p>

        <form id="sf" class="inline-form">
            <input name="sym" placeholder="symbol (GME)" style="text-transform:uppercase">
            <button data-i18n="view.short_interest.btn.lookup" class="primary" type="submit">Lookup</button>
        </form>

        <div class="chart-panel">
            <h2 data-i18n="view.short_interest.h2.watchlist_ranking_sorted_by_short_of_float">Watchlist ranking (sorted by short % of float)</h2>
            <form id="rf" class="inline-form">
                <label><span data-i18n="view.short_interest.label.universe">Universe</span>
                    <select name="wl">
                        <option data-i18n="view.short_interest.opt.all_my_watchlists" value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <button data-i18n="view.short_interest.btn.rank" class="primary" type="submit">Rank</button>
            </form>
            <div id="ranked"></div>
        </div>
    `;
    mount.querySelector('#sf').addEventListener('submit', (e) => {
        e.preventDefault();
        const s = new FormData(e.target).get('sym').trim().toUpperCase();
        if (s) window.location.hash = `short-interest/${encodeURIComponent(s)}`;
    });
    mount.querySelector('#rf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const wid = fd.get('wl') || null;
        const el = mount.querySelector('#ranked');
        if (!el) return;
        el.innerHTML = '<div class="boot">fetching…</div>';
        try {
            const rows = await api.shortRanked(wid);
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#ranked');
            if (elNow) renderRanked(elNow, rows);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#ranked');
            if (elNow) elNow.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderRanked(el, rows) {
    if (!rows.length) { el.innerHTML = '<p data-i18n="view.short_interest.hint.no_symbols_in_this_universe" class="muted">No symbols in this universe.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr><th>#</th><th data-i18n="view.short_interest.th.sym">Sym</th><th data-i18n="view.short_interest.th.shares_short">Shares short</th><th data-i18n="view.short_interest.th.prior_month">Prior month</th><th>Δ</th>
            <th data-i18n="view.short_interest.th.float">% Float</th><th data-i18n="view.short_interest.th.outstanding">% Outstanding</th><th data-i18n="view.short_interest.th.days_to_cover">Days to cover</th><th data-i18n="view.short_interest.th.float_2">Float</th></tr></thead>
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

async function renderSymbol(mount, sym, tok) {
    mount.innerHTML = `
        <h1 class="view-title">// SHORT INTEREST · ${esc(sym)}
            <a class="link small" href="#short-interest">← back</a>
        </h1>
        <div id="ss-cards" class="cards">loading…</div>
        <div class="chart-panel">
            <h2 data-i18n="view.short_interest.h2.finra_reg_sho_daily_short_volume_last_30_sessions">FINRA Reg SHO daily short volume (last 30 sessions)</h2>
            <div id="finra-vol"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.short_interest.h2.finra_short_of_total_volume">FINRA short % of total volume</h2>
            <div id="finra-pct"></div>
        </div>
    `;
    try {
        const [s, days] = await Promise.all([
            api.shortSymbol(sym),
            api.shortFinra(sym, 30).catch(() => []),
        ]);
        if (!viewIsCurrent(tok)) return;
        const changeCls = (s.change_pct ?? 0) >= 0 ? 'neg' : 'pos';
        const cardsEl = mount.querySelector('#ss-cards');
        if (!cardsEl) return;
        cardsEl.innerHTML = `
            <div class="card"><div class="label" data-i18n="view.short_interest.card.shares_short">Shares short</div><div class="value">${compact(s.shares_short)}</div></div>
            <div class="card"><div class="label" data-i18n="view.short_interest.card.prior_month">Prior month</div><div class="value">${compact(s.shares_short_prior)}</div></div>
            <div class="card"><div class="label">Δ vs prior</div>
                <div class="value ${changeCls}">${pctSigned(s.change_pct)}</div></div>
            <div class="card"><div class="label">% of float</div><div class="value">${pct1(s.short_pct_float)}</div></div>
            <div class="card"><div class="label">% of shares out</div><div class="value">${pct1(s.short_pct_outstanding)}</div></div>
            <div class="card"><div class="label" data-i18n="view.short_interest.card.days_to_cover">Days to cover</div><div class="value">${s.short_ratio != null ? s.short_ratio.toFixed(2) : '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.short_interest.card.float">Float</div><div class="value">${compact(s.float)}</div></div>
        `;
        try { applyUiI18n(cardsEl); } catch (_) {}
        const labels = days.map(d => d.date);
        const vols   = days.map(d => Number(d.short_volume));
        const pcts   = days.map(d => Number(d.short_pct));
        const volEl = mount.querySelector('#finra-vol');
        const pctEl = mount.querySelector('#finra-pct');
        if (days.length) {
            if (volEl) barChart(volEl, labels, vols, { color: '#ff2a6d' });
            if (pctEl) barChart(pctEl, labels, pcts, { color: '#b86bff' });
        } else if (volEl) {
            volEl.innerHTML = '<p data-i18n="view.short_interest.hint.no_finra_data_file_may_be_embargoed_or_pending" class="muted">No FINRA data — file may be embargoed or pending.</p>';
        }
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const cardsEl = mount.querySelector('#ss-cards');
        if (cardsEl) cardsEl.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
