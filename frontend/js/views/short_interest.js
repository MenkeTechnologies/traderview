// Short Interest tracker — Yahoo stats + FINRA daily short-volume.
import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n, t } from '../i18n.js';

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
            <input name="sym" data-shortcut="focus_search" placeholder="symbol (GME)" data-i18n-placeholder="view.short_interest.placeholder.symbol" style="text-transform:uppercase">
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
            <div id="ranked-chart" style="width:100%;height:240px;margin-top:14px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.short_interest.h2.dtc_chart">Days-to-cover ranking (squeeze potential vs raw short %)</h2>
            <div id="ranked-dtc-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.short_interest.hint.dtc_chart" class="muted small">Top 20 by short_ratio (shares short ÷ avg daily volume). Orthogonal to short % of float: a 30 %-short name with 1 day-to-cover unwinds easily; a 5 %-short name with 10 days-to-cover is squeeze-prone because shorts can't exit on a single high-volume day.</p>
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
        el.innerHTML = '<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>';
        try {
            const rows = await api.shortRanked(wid);
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#ranked');
            if (elNow) renderRanked(elNow, rows);
            renderRankedChart(rows, mount);
            renderDtcChart(rows, mount);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#ranked');
            if (elNow) elNow.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderRankedChart(rows, mount) {
    const el = mount.querySelector('#ranked-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (rows || []).filter(r => Number.isFinite(r.short_pct_float));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.short_interest.empty_chart">${esc(t('view.short_interest.empty_chart'))}</div>`;
        return;
    }
    const top20 = valid.slice(0, 20);
    const labels = top20.map(r => r.symbol);
    const sf = top20.map(r => r.short_pct_float * 100);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.short_interest.chart.symbol_idx') },
            { label: t('view.short_interest.chart.short_pct_float'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, sf], el);
}

function renderDtcChart(rows, mount) {
    const el = mount.querySelector('#ranked-dtc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (rows || []).filter(r => Number.isFinite(Number(r.short_ratio)) && Number(r.short_ratio) > 0);
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.short_interest.empty_dtc_chart">${esc(t('view.short_interest.empty_dtc_chart'))}</div>`;
        return;
    }
    const sorted = [...valid].sort((a, b) => Number(b.short_ratio) - Number(a.short_ratio)).slice(0, 20);
    const labels = sorted.map(r => r.symbol);
    const ys = sorted.map(r => Number(r.short_ratio));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.short_interest.chart.symbol_idx') },
            { label: t('view.short_interest.chart.dtc'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50,
              values: (_u, splits) => splits.map(v => v.toFixed(1) + 'd') },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderRanked(el, rows) {
    if (!rows.length) { el.innerHTML = '<p data-i18n="view.short_interest.hint.no_symbols_in_this_universe" class="muted">No symbols in this universe.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr><th>#</th><th data-i18n="view.short_interest.th.sym">Sym</th><th data-i18n="view.short_interest.th.shares_short">Shares short</th><th data-i18n="view.short_interest.th.prior_month">Prior month</th><th>Δ</th>
            <th data-i18n="view.short_interest.th.float">% Float</th><th data-i18n="view.short_interest.th.outstanding">% Outstanding</th><th data-i18n="view.short_interest.th.days_to_cover">Days to cover</th><th data-i18n="view.short_interest.th.float_2">Float</th></tr></thead>
        <tbody>${rows.map((r, i) => `
            <tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
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
        <div id="ss-cards" class="cards"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
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
            <div class="card"><div class="label" data-i18n="view.short_interest.card.delta_vs_prior">Δ vs prior</div>
                <div class="value ${changeCls}">${pctSigned(s.change_pct)}</div></div>
            <div class="card"><div class="label" data-i18n="view.short_interest.card.pct_float">% of float</div><div class="value">${pct1(s.short_pct_float)}</div></div>
            <div class="card"><div class="label" data-i18n="view.short_interest.card.pct_shares_out">% of shares out</div><div class="value">${pct1(s.short_pct_outstanding)}</div></div>
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
