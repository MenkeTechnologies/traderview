// Sentiment-tagged news — searchable history + per-symbol feed.
import { api } from '../api.js';
import { esc } from '../util.js';
import { on as onWsEvent } from '../ws.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let timer = null;
let wsUnsub = null;
let lastQuery = { mode: 'recent', sym: '', q: '' };

export async function renderNews(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.news.h1.news" class="view-title">// NEWS</h1>
        <p data-i18n="view.news.hint.yahoo_headlines_polled_per_watchlist_symbol_every_" class="muted small">Yahoo headlines polled per watchlist symbol every 5 minutes,
            scored with the same WSB-aware sentiment lexicon used for social feeds, and
            indexed for full-text search via Postgres tsvector / websearch_to_tsquery.
            Color stripe on each row maps to sentiment: red (negative) → grey → green (positive).</p>

        <div class="chart-panel">
            <form id="n-form" class="inline-form">
                <select name="mode">
                    <option data-i18n="view.news.opt.recent_global" value="recent">recent (global)</option>
                    <option data-i18n="view.news.opt.by_symbol" value="symbol">by symbol</option>
                    <option data-i18n="view.news.opt.full_text_search" value="search">full-text search</option>
                </select>
                <input name="value" data-shortcut="focus_search" placeholder="symbol or query"
                       data-i18n-placeholder="view.news.placeholder.value" style="min-width:200px;">
                <label><span data-i18n="view.news.label.limit">Limit</span>
                    <input name="limit" type="number" min="10" max="200" value="40" style="width:80px;"></label>
                <button data-i18n="view.news.btn.fetch" class="primary" type="submit">Fetch</button>
                <button data-i18n="view.news.btn.poll_now" type="button" class="btn" id="n-poll-now">Poll now</button>
                <span id="n-status" class="muted small"></span>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.news.h2.sentiment_chart">Sentiment by item (most recent first)</h2>
            <div id="n-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.news.h2.symbol_sentiment_chart">Average sentiment per symbol</h2>
            <div id="n-symbol-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.news.hint.symbol_sentiment" class="muted small">Mean sentiment across all loaded items per symbol. Yellow dashed = neutral. Reveals which tickers have the tape leaning bullish vs bearish independent of item count.</p>
        </div>

        <div id="n-list"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
    `;
    mount.querySelector('#n-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        lastQuery = {
            mode: fd.get('mode'),
            sym: fd.get('value').trim().toUpperCase(),
            q: fd.get('value').trim(),
            limit: Number(fd.get('limit')) || 40,
        };
        await refresh(mount, tok);
    });
    mount.querySelector('#n-poll-now').addEventListener('click', async () => {
        const status = mount.querySelector('#n-status');
        if (status) status.textContent = t('view.news.status.polling_watchlists');
        try {
            const s = await api.newsPollNow();
            if (!viewIsCurrent(tok)) return;
            const s2 = mount.querySelector('#n-status');
            if (s2) s2.textContent = t('view.news.status.result', { symbols: s.symbols_polled, inserted: s.inserted });
            await refresh(mount, tok);
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const s2 = mount.querySelector('#n-status');
            if (s2) s2.textContent = t('common.error', { err: e.message });
        }
    });

    if (timer) clearInterval(timer);
    timer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(timer); timer = null; return; }
        refresh(mount, tok);
    }, 120_000);
    if (wsUnsub) wsUnsub();
    wsUnsub = onWsEvent('news', () => { if (viewIsCurrent(tok)) refresh(mount, tok); });

    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#news')) {
            clearInterval(timer); timer = null;
            if (wsUnsub) { wsUnsub(); wsUnsub = null; }
        }
    }, { once: true });

    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    const list = mount.querySelector('#n-list');
    if (!list) return;
    const limit = lastQuery.limit || 40;
    try {
        let items;
        if (lastQuery.mode === 'symbol' && lastQuery.sym) {
            items = await api.newsBySymbol(lastQuery.sym, limit);
        } else if (lastQuery.mode === 'search' && lastQuery.q) {
            items = await api.newsSearch(lastQuery.q, limit);
        } else {
            items = await api.newsRecent(limit);
        }
        if (!viewIsCurrent(tok)) return;
        const list2 = mount.querySelector('#n-list');
        if (list2) renderList(list2, items);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const list2 = mount.querySelector('#n-list');
        if (list2) list2.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderList(el, items) {
    if (!items.length) { el.innerHTML = '<p data-i18n="view.news.hint.no_items" class="muted small">no items</p>'; return; }
    el.innerHTML = items.map(n => row(n)).join('');
    renderSentimentChart(items);
    renderSymbolSentimentChart(items);
}

function renderSymbolSentimentChart(items) {
    const el = document.getElementById('n-symbol-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const bySym = new Map();
    for (const n of (items || [])) {
        const s = Number(n.sentiment);
        if (!Number.isFinite(s) || !n.symbol) continue;
        const cur = bySym.get(n.symbol) || { sum: 0, n: 0 };
        cur.sum += s; cur.n += 1;
        bySym.set(n.symbol, cur);
    }
    if (bySym.size < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.news.empty_symbol_chart">${esc(t('view.news.empty_symbol_chart'))}</div>`;
        return;
    }
    const pairs = Array.from(bySym.entries())
        .map(([sym, c]) => [sym, c.sum / c.n])
        .sort((a, b) => b[1] - a[1]);
    const labels = pairs.map(([s]) => s);
    const ys = pairs.map(([, v]) => v);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.news.chart.symbol_idx') },
            { label: t('view.news.chart.avg_sentiment'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.news.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderSentimentChart(items) {
    const el = document.getElementById('n-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const scored = (items || []).filter(n => Number.isFinite(Number(n.sentiment)));
    if (scored.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.news.empty_chart">${esc(t('view.news.empty_chart'))}</div>`;
        return;
    }
    const ys = scored.map(n => Number(n.sentiment));
    const xs = ys.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.news.chart.item_idx') },
            { label: t('view.news.chart.sentiment'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.news.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 } ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function sentimentBar(s) {
    if (s == null) return '<span style="display:inline-block;width:8px;height:18px;background:#444;margin-right:8px;"></span>';
    const v = Math.max(-1, Math.min(1, s));
    const color = v > 0.1 ? '#7af0a8' : v < -0.1 ? '#ff1f7a' : '#9aa0c8';
    return `<span title="${t('view.news.tip.sentiment', { score: v.toFixed(2) })}" style="display:inline-block;width:8px;height:18px;background:${color};margin-right:8px;vertical-align:middle;"></span>`;
}

function row(n) {
    const when = n.published_at || n.fetched_at;
    const ago = relativeTime(when);
    const link = n.link
        ? `<a href="${esc(n.link)}" target="_blank" rel="noopener">${esc(n.title)}</a>`
        : esc(n.title);
    return `<div class="chart-panel" style="padding:8px 10px;margin-bottom:6px;"
                 data-context-scope="symbol-row" data-symbol="${esc(n.symbol)}">
        <div style="display:flex;align-items:flex-start;">
            ${sentimentBar(n.sentiment)}
            <div style="flex:1 1 auto;min-width:0;">
                <div style="font-size:13px;">${link}</div>
                <div class="muted small">
                    <strong>${esc(n.symbol)}</strong> ·
                    ${esc(n.publisher || t('common.unknown'))} · ${esc(ago)}
                    ${n.sentiment != null ? esc(t('view.news.row.sentiment_suffix', { score: n.sentiment.toFixed(2) })) : ''}
                </div>
            </div>
        </div>
    </div>`;
}

function relativeTime(iso) {
    if (!iso) return '';
    const ms = new Date(iso).getTime();
    const diff = (Date.now() - ms) / 1000;
    if (diff < 60)   return t('common.ago.s', { n: Math.floor(diff) });
    if (diff < 3600) return t('common.ago.m', { n: Math.floor(diff / 60) });
    if (diff < 86400) return t('common.ago.h', { n: Math.floor(diff / 3600) });
    return t('common.ago.d', { n: Math.floor(diff / 86400) });
}
