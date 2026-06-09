// Top News — Finnhub `/news?category=` consumer. Broad market news strip
// useful as a side feed; complements the per-symbol catalysts firehose.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const CATEGORIES = [
    { value: 'general',  key: 'view.top_news.category.general' },
    { value: 'forex',    key: 'view.top_news.category.forex' },
    { value: 'crypto',   key: 'view.top_news.category.crypto' },
    { value: 'merger',   key: 'view.top_news.category.merger' },
];

let state = { category: 'general' };

export async function renderTopNews(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.top_news.h1.title">// TOP NEWS</span></h1>
        <p class="muted small" data-i18n="view.top_news.hint.intro">
            Broad market news from Finnhub. Category filter switches between
            general macro, FX, crypto, and M&amp;A streams.
        </p>

        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.top_news.label.category">Category</span>
                    <select id="tn-category">${CATEGORIES.map(c =>
                        `<option value="${c.value}" ${c.value === state.category ? 'selected' : ''}>${esc(t(c.key))}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" id="tn-refresh" type="button"
                    data-i18n="view.top_news.btn.refresh">Refresh</button>
            </div>
            <div id="tn-list" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('tn-category').addEventListener('change', e => {
        state.category = e.target.value;
        void load(tok);
    });
    document.getElementById('tn-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

async function load(tok) {
    const el = document.getElementById('tn-list');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const items = await api.finnhubGeneralNews(state.category);
        if (!viewIsCurrent(tok)) return;
        renderList(Array.isArray(items) ? items : []);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.top_news.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.top_news.toast.load_failed'), { level: 'error' });
    }
}

function renderList(items) {
    const el = document.getElementById('tn-list');
    if (!el) return;
    if (!items.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.top_news.empty">No news in this category right now.</p>`;
        return;
    }
    // Finnhub returns objects like {category, datetime, headline, id, image,
    // related, source, summary, url}. Sort newest-first defensively.
    const sorted = [...items].sort((a, b) => (b.datetime || 0) - (a.datetime || 0));
    el.innerHTML = sorted.map(n => `
        <div class="news-item" style="border-bottom:1px solid var(--border);padding:10px 0">
            <a href="${esc(n.url || '#')}" target="_blank" rel="noopener noreferrer" style="font-weight:600">
                ${esc(n.headline || '(no title)')}
            </a>
            <div class="muted small" style="margin-top:2px">
                ${esc(n.source || '—')}
                ${n.datetime ? '· ' + new Date(n.datetime * 1000).toLocaleString(undefined, { hour12: false }) : ''}
                ${n.related ? '· ' + esc(n.related.split(',').slice(0, 5).join(' ')) : ''}
            </div>
            ${n.summary ? `<p class="muted small" style="margin-top:4px">${esc(n.summary.slice(0, 280))}${n.summary.length > 280 ? '…' : ''}</p>` : ''}
        </div>
    `).join('');
}
