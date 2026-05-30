// Sentiment-as-a-feed — WSB + StockTwits stream + delta ranking + per-symbol
// time-series. Polls every 60s.

import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc, fmt, fmtDateTime } from '../util.js';
import { on as onWsEvent } from '../ws.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t, applyUiI18n } from '../i18n.js';

let timer = null;
let wsUnsub = null;

export async function renderSentiment(mount, _state, symbol) {
    const tok = currentViewToken();
    if (symbol) return renderSymbol(mount, symbol.toUpperCase(), tok);

    mount.innerHTML = `
        <h1 data-i18n="view.sentiment.h1.sentiment_wsb_stocktwits" class="view-title">// SENTIMENT — WSB + STOCKTWITS</h1>
        <p data-i18n="view.sentiment.hint.lexicon_scorer_wsb_aware_over_reddit_r_wallstreetb" class="muted small">
            Lexicon scorer (WSB-aware) over Reddit r/wallstreetbets new posts +
            StockTwits symbol streams. Auto-polls every 60s server-side; this
            view also refreshes every 60s.
        </p>

        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.sentiment.label.window">Window</span>
                    <select id="hours">
                        <option data-i18n="view.sentiment.opt.last_hour" value="1">last hour</option>
                        <option data-i18n="view.sentiment.opt.last_4h" value="4">last 4h</option>
                        <option data-i18n="view.sentiment.opt.last_24h" value="24" selected>last 24h</option>
                        <option data-i18n="view.sentiment.opt.last_7d" value="168">last 7d</option>
                    </select>
                </label>
                <button data-i18n="view.sentiment.btn.poll_now" class="primary" id="poll-now">Poll now</button>
                <span class="muted" id="poll-status"></span>
            </div>
        </div>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2><span data-i18n="view.sentiment.h2.top_sentiment">Top sentiment</span> <span style="color:var(--green)" data-i18n="view.sentiment.h2.deltas_up">deltas ↑</span></h2>
                <div id="top-up"></div>
            </div>
            <div class="chart-panel">
                <h2><span data-i18n="view.sentiment.h2.top_sentiment">Top sentiment</span> <span style="color:var(--red)" data-i18n="view.sentiment.h2.deltas_down">deltas ↓</span></h2>
                <div id="top-down"></div>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.sentiment.h2.most_mentioned_volume">Most-mentioned (volume)</h2>
            <div id="top-volume"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.sentiment.h2.delta_chart">Top sentiment Δ by symbol</h2>
            <div id="sent-delta-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.sentiment.h2.mention_chart">Top mention volume by symbol</h2>
            <div id="sent-mention-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.sentiment.hint.mention_chart" class="muted small">Raw mention count per symbol for the top 30 most-mentioned in this window. Orthogonal to sentiment Δ: reveals which names have the heaviest chatter regardless of direction.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.sentiment.h2.live_feed">Live feed</h2>
            <div id="feed"></div>
        </div>
    `;
    mount.querySelector('#hours').addEventListener('change', () => refresh(mount, tok));
    mount.querySelector('#poll-now').addEventListener('click', async () => {
        const status = mount.querySelector('#poll-status');
        if (status) status.textContent = t('common.status.polling');
        try {
            const r = await api.sentimentPollNow();
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#poll-status');
            if (status2) status2.textContent = t('view.sentiment.status.result', { wsb: r.wsb_inserted, stocktwits: r.stocktwits_inserted });
            await refresh(mount, tok);
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#poll-status');
            if (status2) status2.textContent = t('common.error', { err: e.message });
        }
    });

    if (timer) clearInterval(timer);
    timer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(timer); timer = null; return; }
        refresh(mount, tok);
    }, 60_000);

    if (wsUnsub) wsUnsub();
    wsUnsub = onWsEvent('sentiment', () => { if (viewIsCurrent(tok)) refresh(mount, tok); });

    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#sentiment')) {
            clearInterval(timer); timer = null;
            if (wsUnsub) { wsUnsub(); wsUnsub = null; }
        }
    }, { once: true });
    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    const hoursEl = mount.querySelector('#hours');
    if (!hoursEl) return;
    const hours = Number(hoursEl.value);
    const [ranked, feed] = await Promise.all([
        api.sentimentRanked(hours, 50).catch(() => []),
        api.sentimentFeed(80).catch(() => []),
    ]);
    if (!viewIsCurrent(tok)) return;
    // Split by direction of sentiment delta.
    const ups   = ranked.filter(r => Number(r.sentiment_delta) > 0)
                        .sort((a, b) => Number(b.sentiment_delta) - Number(a.sentiment_delta))
                        .slice(0, 20);
    const downs = ranked.filter(r => Number(r.sentiment_delta) < 0)
                        .sort((a, b) => Number(a.sentiment_delta) - Number(b.sentiment_delta))
                        .slice(0, 20);
    const byVol = [...ranked].sort((a, b) => Number(b.mention_count) - Number(a.mention_count))
                             .slice(0, 25);

    const upEl = mount.querySelector('#top-up');
    const downEl = mount.querySelector('#top-down');
    const volEl = mount.querySelector('#top-volume');
    const feedEl = mount.querySelector('#feed');
    if (upEl) upEl.innerHTML   = rankedTable(ups, 'up');
    if (downEl) downEl.innerHTML = rankedTable(downs, 'down');
    if (volEl) volEl.innerHTML = volumeTable(byVol);
    if (feedEl) feedEl.innerHTML = feedTable(feed);
    renderDeltaChart(ranked);
    renderMentionChart(ranked);
}

function renderMentionChart(ranked) {
    const el = document.getElementById('sent-mention-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const top = [...(ranked || [])]
        .filter(r => Number.isFinite(Number(r.mention_count)))
        .sort((a, b) => Number(b.mention_count) - Number(a.mention_count))
        .slice(0, 30);
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.sentiment.empty_mention_chart">${esc(t('view.sentiment.empty_mention_chart'))}</div>`;
        return;
    }
    const labels = top.map(r => r.symbol);
    const counts = top.map(r => Number(r.mention_count));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.sentiment.chart.symbol_idx') },
            { label: t('view.sentiment.chart.mentions'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, counts], el);
}

function renderDeltaChart(ranked) {
    const el = document.getElementById('sent-delta-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const top = [...(ranked || [])]
        .filter(r => Number.isFinite(Number(r.sentiment_delta)))
        .sort((a, b) => Math.abs(Number(b.sentiment_delta)) - Math.abs(Number(a.sentiment_delta)))
        .slice(0, 30);
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.sentiment.empty_chart">${esc(t('view.sentiment.empty_chart'))}</div>`;
        return;
    }
    top.sort((a, b) => Number(b.sentiment_delta) - Number(a.sentiment_delta));
    const labels = top.map(r => r.symbol);
    const deltas = top.map(r => Number(r.sentiment_delta));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.sentiment.chart.symbol_idx') },
            { label: t('view.sentiment.chart.delta'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.sentiment.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, deltas, zero], el);
}

function rankedTable(rows, dir) {
    if (!rows.length) return '<p data-i18n="view.sentiment.hint.no_symbols_match_in_this_window_yet" class="muted">No symbols match in this window yet.</p>';
    return `<table class="trades">
        <thead><tr><th>#</th><th data-i18n="view.sentiment.th.sym">Sym</th><th data-i18n="view.sentiment.th.sent">Sent</th><th data-i18n="view.sentiment.th.sent_2">Δ Sent</th><th data-i18n="view.sentiment.th.mentions">Mentions</th><th data-i18n="view.sentiment.th.vol">Δ Vol</th></tr></thead>
        <tbody>${rows.map((r, i) => {
            const sd = Number(r.sentiment_delta);
            const sc = Number(r.avg_sentiment);
            const sdCls = sd >= 0 ? 'pos' : 'neg';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td>${i + 1}</td>
                <td><a href="#sentiment/${encodeURIComponent(r.symbol)}">${esc(r.symbol)}</a></td>
                <td class="${sc >= 0 ? 'pos' : 'neg'}">${sc >= 0 ? '+' : ''}${sc.toFixed(2)}</td>
                <td class="${sdCls}">${sd >= 0 ? '+' : ''}${sd.toFixed(2)}</td>
                <td>${r.mention_count}</td>
                <td class="${Number(r.count_delta) >= 0 ? 'pos' : 'neg'}">${Number(r.count_delta) >= 0 ? '+' : ''}${r.count_delta}</td>
            </tr>`;
        }).join('')}</tbody></table>`;
}

function volumeTable(rows) {
    if (!rows.length) return '<p data-i18n="view.sentiment.hint.no_mentions_yet" class="muted">No mentions yet.</p>';
    return `<table class="trades">
        <thead><tr><th>#</th><th data-i18n="view.sentiment.th.sym_2">Sym</th><th data-i18n="view.sentiment.th.mentions_2">Mentions</th><th data-i18n="view.sentiment.th.avg_sent">Avg Sent</th><th data-i18n="view.sentiment.th.sent_3">Δ Sent</th></tr></thead>
        <tbody>${rows.map((r, i) => {
            const sc = Number(r.avg_sentiment);
            const sd = Number(r.sentiment_delta);
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td>${i + 1}</td>
                <td><a href="#sentiment/${encodeURIComponent(r.symbol)}">${esc(r.symbol)}</a></td>
                <td><strong>${r.mention_count}</strong></td>
                <td class="${sc >= 0 ? 'pos' : 'neg'}">${sc.toFixed(2)}</td>
                <td class="${sd >= 0 ? 'pos' : 'neg'}">${sd >= 0 ? '+' : ''}${sd.toFixed(2)}</td>
            </tr>`;
        }).join('')}</tbody></table>`;
}

function feedTable(items) {
    if (!items.length) return '<p data-i18n="view.sentiment.hint.no_posts_cached_yet_try_poll_now" class="muted">No posts cached yet — try "Poll now".</p>';
    return `<table class="trades">
        <thead><tr><th data-i18n="view.sentiment.th.when">When</th><th data-i18n="view.sentiment.th.source">Source</th><th data-i18n="view.sentiment.th.sym_3">Sym</th><th data-i18n="view.sentiment.th.sent_4">Sent</th><th data-i18n="view.sentiment.th.author">Author</th><th data-i18n="view.sentiment.th.snippet">Snippet</th></tr></thead>
        <tbody>${items.map(m => {
            const sc = Number(m.sentiment);
            const cls = sc >= 0.1 ? 'pos' : sc <= -0.1 ? 'neg' : '';
            return `<tr>
                <td>${fmtDateTime(m.posted_at)}</td>
                <td><span class="tape-sym">${esc(m.source)}</span></td>
                <td><a href="#sentiment/${encodeURIComponent(m.symbol)}">${esc(m.symbol)}</a></td>
                <td class="${cls}">${sc >= 0 ? '+' : ''}${sc.toFixed(2)}</td>
                <td>${esc(m.author || '')}</td>
                <td>${m.url ? `<a href="${esc(m.url)}" target="_blank" rel="noopener noreferrer">${esc(m.snippet || '').slice(0, 180)}</a>` : esc(m.snippet || '').slice(0, 180)}</td>
            </tr>`;
        }).join('')}</tbody></table>`;
}

async function renderSymbol(mount, sym, tok) {
    mount.innerHTML = `
        <h1 class="view-title">// SENTIMENT · ${esc(sym)}
            <a class="link small" href="#sentiment">← back</a>
        </h1>
        <div id="sym-cards" class="cards"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>

        <div class="chart-panel">
            <h2 data-i18n="view.sentiment.h2.hourly_mention_volume_last_7d">Hourly mention volume (last 7d)</h2>
            <div id="sent-vol-chart"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.sentiment.h2.hourly_avg_sentiment">Hourly avg sentiment</h2>
            <div id="sent-score-chart"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.sentiment.h2.recent_mentions">Recent mentions</h2>
            <div id="sent-list"></div>
        </div>
    `;
    try {
        const [series, mentions] = await Promise.all([
            api.sentimentSeries(sym, 168),
            api.sentimentForSymbol(sym, 168, 200),
        ]);
        if (!viewIsCurrent(tok)) return;

        // Aggregate per hour across sources.
        const buckets = new Map();
        for (const b of series) {
            const k = b.bucket_hour;
            const cur = buckets.get(k) || { count: 0, weighted: 0 };
            cur.count += Number(b.mention_count);
            cur.weighted += Number(b.avg_sentiment) * Number(b.mention_count);
            buckets.set(k, cur);
        }
        const sorted = Array.from(buckets.entries())
            .sort((a, b) => new Date(a[0]) - new Date(b[0]));
        const labels = sorted.map(([k]) => new Date(k).toLocaleString(undefined, { hour: '2-digit', day: '2-digit', month: '2-digit' }));
        const counts = sorted.map(([, v]) => v.count);
        const scores = sorted.map(([, v]) => v.count > 0 ? v.weighted / v.count : 0);
        const totalCount = counts.reduce((a, b) => a + b, 0);
        const avgScore = totalCount > 0
            ? scores.reduce((a, b, i) => a + b * counts[i], 0) / totalCount : 0;

        const cardsEl = mount.querySelector('#sym-cards');
        if (cardsEl) {
            cardsEl.innerHTML = `
            <div class="card"><div class="label" data-i18n="view.sentiment.card.mentions_7d">Mentions (7d)</div><div class="value">${totalCount}</div></div>
            <div class="card"><div class="label" data-i18n="view.sentiment.card.avg_sentiment">Avg sentiment</div>
                <div class="value ${avgScore >= 0 ? 'pos' : 'neg'}">${avgScore >= 0 ? '+' : ''}${avgScore.toFixed(2)}</div></div>
            <div class="card"><div class="label" data-i18n="view.sentiment.card.hours_covered">Hours covered</div><div class="value">${sorted.length}</div></div>
        `;
            try { applyUiI18n(cardsEl); } catch (_) {}
        }

        const volChartEl = mount.querySelector('#sent-vol-chart');
        const scoreChartEl = mount.querySelector('#sent-score-chart');
        if (volChartEl) barChart(volChartEl, labels, counts, { color: '#00e5ff' });
        if (scoreChartEl) barChart(scoreChartEl, labels, scores, { color: '#b86bff' });

        const listEl = mount.querySelector('#sent-list');
        if (listEl) listEl.innerHTML = feedTable(mentions);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const cardsEl = mount.querySelector('#sym-cards');
        if (cardsEl) cardsEl.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
    void fmt;
}
