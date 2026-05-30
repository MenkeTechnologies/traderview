// Live tape — Zendoo-style streaming feed: news + scanner pings + sector
// snapshot, refreshed every 30s.
import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let timer = null;

export async function renderTape(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.tape.h1.live_tape" class="view-title">// LIVE TAPE</h1>
        <p data-i18n="view.tape.hint.auto_refreshing_news_sector_snapshot_from_your_wat" class="muted small">Auto-refreshing news + sector snapshot from your watchlist universe. Updates every 30 seconds.</p>
        <div class="panel-grid">
            <div class="chart-panel" style="grid-column: 1 / -1">
                <h2 data-i18n="view.tape.h2.news_feed">News feed</h2>
                <div id="tape-news" data-i18n="common.loading">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.tape.h2.sectors_right_now">Sectors right now</h2>
                <div id="tape-sectors" data-i18n="common.loading">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.tape.h2.watchlist_quotes">Watchlist quotes</h2>
                <div id="tape-quotes" data-i18n="common.loading">loading…</div>
            </div>
            <div class="chart-panel" style="grid-column: 1 / -1">
                <h2 data-i18n="view.tape.h2.change_chart">Watchlist change % snapshot</h2>
                <div id="tape-chart" style="width:100%;height:240px"></div>
            </div>
            <div class="chart-panel" style="grid-column: 1 / -1">
                <h2 data-i18n="view.tape.h2.news_chart">News count per symbol (this refresh)</h2>
                <div id="tape-news-chart" style="width:100%;height:220px"></div>
                <p data-i18n="view.tape.hint.news_chart" class="muted small">How many news items each symbol generated in this refresh. Orthogonal to price action: a flat-tape symbol can be the day's news leader; a big mover can have zero news (technical-only move).</p>
            </div>
        </div>
    `;
    if (timer) clearInterval(timer);
    timer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(timer); timer = null; return; }
        refresh(mount, tok);
    }, 30_000);
    await refresh(mount, tok);
    // Stop polling when leaving the view.
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#tape')) {
            clearInterval(timer); timer = null;
        }
    }, { once: true });
}

async function refresh(mount, tok) {
    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    const symbols = new Set();
    for (const w of lists) {
        for (const s of await api.watchlistSymbols(w.id)) symbols.add(s);
        if (!viewIsCurrent(tok)) return;
    }
    const syms = Array.from(symbols).slice(0, 12);

    // News: pull a few items per symbol, flatten, sort by time.
    const allNews = [];
    for (const sym of syms) {
        try {
            const items = await api.symbolNews(sym, 3);
            if (!viewIsCurrent(tok)) return;
            for (const n of items) allNews.push({ ...n, symbol: sym });
        } catch (_) {}
    }
    allNews.sort((a, b) => (b.provider_publish_time || 0) - (a.provider_publish_time || 0));
    const newsEl = mount.querySelector('#tape-news');
    if (newsEl) newsEl.innerHTML = allNews.length
        ? allNews.slice(0, 40).map(n => `
            <div class="news-item" data-context-scope="symbol-row" data-symbol="${esc(n.symbol)}">
                <a href="${esc(n.link || '#')}" target="_blank" rel="noopener noreferrer">
                    <span class="tape-sym">${esc(n.symbol)}</span> ${esc(n.title || '(no title)')}
                </a>
                <div class="meta">${esc(n.publisher || '')} ${n.provider_publish_time
                    ? '· ' + new Date(n.provider_publish_time * 1000).toLocaleString(undefined, { hour12: false })
                    : ''}</div>
            </div>`).join('')
        : '<p data-i18n="view.tape.hint.no_news_in_this_universe_yet" class="muted">No news in this universe yet.</p>';

    // Sectors
    try {
        const sectors = await api.sectors();
        if (!viewIsCurrent(tok)) return;
        const secEl = mount.querySelector('#tape-sectors');
        if (secEl) secEl.innerHTML = `<table class="trades">
            ${sectors.map(s => `<tr>
                <td>${esc(s.label)}</td>
                <td><a href="#research/${encodeURIComponent(s.sector)}">${esc(s.sector)}</a></td>
                <td class="${Number(s.change_pct) >= 0 ? 'pos' : 'neg'}">${Number(s.change_pct) >= 0 ? '+' : ''}${Number(s.change_pct).toFixed(2)}%</td>
            </tr>`).join('')}
        </table>`;
    } catch (_) {}

    // Quotes
    const quotes = [];
    for (const sym of syms) {
        try {
            quotes.push(await api.quote(sym));
            if (!viewIsCurrent(tok)) return;
        } catch (_) {}
    }
    const quotesEl = mount.querySelector('#tape-quotes');
    if (quotesEl) quotesEl.innerHTML = quotes.length ? `<table class="trades">
        ${quotes.map(q => `<tr data-context-scope="symbol-row" data-symbol="${esc(q.symbol)}">
            <td><a href="#research/${encodeURIComponent(q.symbol)}">${esc(q.symbol)}</a></td>
            <td>${Number(q.price).toFixed(2)}</td>
            <td class="${(q.change_pct ?? 0) >= 0 ? 'pos' : 'neg'}">${q.change_pct != null ? (q.change_pct >= 0 ? '+' : '') + q.change_pct.toFixed(2) + '%' : '—'}</td>
        </tr>`).join('')}
    </table>` : '<p data-i18n="view.tape.hint.add_symbols_to_a_watchlist_first" class="muted">Add symbols to a watchlist first.</p>';

    renderChangeChart(quotes);
    renderNewsCountChart(allNews);
    void fmtDateTime;
}

function renderNewsCountChart(news) {
    const el = document.getElementById('tape-news-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const buckets = new Map();
    for (const n of (news || [])) {
        if (!n || !n.symbol) continue;
        buckets.set(n.symbol, (buckets.get(n.symbol) || 0) + 1);
    }
    if (buckets.size < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.tape.empty_news_chart">${esc(t('view.tape.empty_news_chart'))}</div>`;
        return;
    }
    const sorted = [...buckets.entries()].sort((a, b) => b[1] - a[1]).slice(0, 25);
    const labels = sorted.map(([s]) => s);
    const ys = sorted.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.tape.chart.symbol_idx') },
            { label: t('view.tape.chart.news_count'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderChangeChart(quotes) {
    const el = document.getElementById('tape-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (quotes || []).filter(q => q && Number.isFinite(Number(q.change_pct)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.tape.empty_chart">${esc(t('view.tape.empty_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => Number(b.change_pct) - Number(a.change_pct));
    const labels = valid.map(q => q.symbol);
    const ys = valid.map(q => Number(q.change_pct));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.tape.chart.symbol_idx') },
            { label: t('view.tape.chart.change_pct'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.tape.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}
