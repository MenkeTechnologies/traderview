// News Sentiment per symbol — Finnhub /news-sentiment.
// Aggregate bullish/bearish scores + buzz vs peers.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderNewsSentiment(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.news_sentiment.h1.title">// NEWS SENTIMENT</span></h1>
        <p class="muted small" data-i18n="view.news_sentiment.hint.intro">
            Finnhub /news-sentiment — aggregate bullish/bearish percentages + sector buzz.
            Sentiment + volume burst = momentum confirmation signal.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="nsm-form">
                <label><span data-i18n="view.news_sentiment.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="TSLA" required></label>
                <button class="primary" type="submit" data-i18n="view.news_sentiment.btn.load">Load</button>
            </form>
            <div id="nsm-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('nsm-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('nsm-result');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.symbolNewsSentiment(state.symbol);
        if (!viewIsCurrent(tok)) return;
        if (!data || typeof data !== 'object' || !Object.keys(data).length) {
            el.innerHTML = `<p class="muted" data-i18n="view.news_sentiment.empty">No sentiment data.</p>`;
            return;
        }
        const buzz = data.buzz || {};
        const sent = data.sentiment || {};
        const bull = Number(sent.bullishPercent || 0);
        const bear = Number(sent.bearishPercent || 0);
        const score = Number(data.companyNewsScore || 0);
        const scoreCls = score >= 0.6 ? 'pos' : score <= 0.4 ? 'neg' : '';
        el.innerHTML = `
            <div class="cards">
                <div class="card ${scoreCls}">
                    <div class="label" data-i18n="view.news_sentiment.card.score">News score</div>
                    <div class="value">${score.toFixed(2)}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.news_sentiment.card.bullish">Bullish</div>
                    <div class="value">${(bull * 100).toFixed(0)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.news_sentiment.card.bearish">Bearish</div>
                    <div class="value">${(bear * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.news_sentiment.card.weekly_avg">Weekly buzz</div>
                    <div class="value">${(buzz.weeklyAverage ?? 0).toFixed(1)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.news_sentiment.card.articles">Articles</div>
                    <div class="value">${buzz.articlesInLastWeek ?? '—'}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.news_sentiment.card.buzz">Buzz score</div>
                    <div class="value">${(buzz.buzz ?? 0).toFixed(2)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.news_sentiment.card.sector_avg">Sector avg score</div>
                    <div class="value">${data.sectorAverageNewsScore != null ? Number(data.sectorAverageNewsScore).toFixed(2) : '—'}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.news_sentiment.card.sector_bullish">Sector avg bullish</div>
                    <div class="value">${data.sectorAverageBullishPercent != null ? (Number(data.sectorAverageBullishPercent) * 100).toFixed(0) + '%' : '—'}</div>
                </div>
            </div>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.news_sentiment.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.news_sentiment.toast.failed'), { level: 'error' });
    }
}
