// Sector-aggregation heatmap. Computes the 11 SPDR ETFs and renders a
// color-coded grid so the user can spot which sectors are flashing
// buy / sell at a glance.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t, applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent, routeIs } from '../app.js';

let timer = null;

export async function renderRecommendationSectors(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.rec_sectors.h1" class="view-title">// SECTOR HEATMAP — Buy/Sell verdicts</h1>
        <p class="muted small" data-i18n="view.rec_sectors.subtitle">
            Composite verdict for each SPDR sector ETF. Same algorithm as the per-symbol
            recommendation, recomputed in the background every few hours.
        </p>
        <div class="chart-panel">
            <div id="rs-sectors-body">
                <span class="tv-spinner-inline" role="status" aria-label="loading"></span>
            </div>
        </div>
    `;
    try { applyUiI18n(mount); } catch (_) {}

    const reload = async () => {
        const rows = await api.recommendationSectors().catch(() => []);
        if (!viewIsCurrent(tok)) return;
        renderBody(mount.querySelector('#rs-sectors-body'), rows);
    };
    await reload();
    if (timer) clearInterval(timer);
    timer = setInterval(() => { if (viewIsCurrent(tok)) reload(); }, 5 * 60_000);
    window.addEventListener('hashchange', () => {
        if (!routeIs('recommendation-sectors')) { clearInterval(timer); timer = null; }
    }, { once: true });
}

function renderBody(el, rows) {
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<div class="boot muted">${esc(t('common.empty.no_data'))}</div>`;
        return;
    }
    el.innerHTML = `
        <div class="rs-sector-grid">
            ${rows.map(r => renderTile(r)).join('')}
        </div>
    `;
}

function renderTile(entry) {
    if (!entry.recommendation) {
        return `
            <div class="rs-sector-tile rs-sector-err">
                <div class="rs-sector-name">${esc(entry.name)}</div>
                <div class="rs-sector-ticker">${esc(entry.ticker)}</div>
                <div class="rs-sector-err-msg muted small">${esc(entry.error || 'n/a')}</div>
            </div>
        `;
    }
    const r = entry.recommendation;
    const verdictCls = verdictClass(r.verdict);
    const stars = '★'.repeat(r.stars);
    const upside = (r.upside_pct >= 0 ? '+' : '') + r.upside_pct.toFixed(1) + '%';
    const upsideCls = r.upside_pct > 0 ? 'pos' : r.upside_pct < 0 ? 'neg' : '';
    return `
        <div class="rs-sector-tile ${verdictCls}">
            <div class="rs-sector-head">
                <a class="rs-sector-ticker" href="#research/${encodeURIComponent(entry.ticker)}">${esc(entry.ticker)}</a>
                <span class="rs-sector-stars">${stars}</span>
            </div>
            <div class="rs-sector-name muted small">${esc(entry.name)}</div>
            <div class="rs-sector-verdict">${verdictLabel(r.verdict)}</div>
            <div class="rs-sector-stats">
                <span class="rs-sector-score">${Math.round(r.score)}</span>
                <span class="rs-sector-upside ${upsideCls}">${upside}</span>
            </div>
        </div>
    `;
}

function verdictLabel(v) {
    return ({
        strong_buy: 'STRONG BUY',
        buy: 'BUY',
        hold: 'HOLD',
        sell: 'SELL',
        strong_sell: 'STRONG SELL',
    })[v] || String(v).toUpperCase();
}

function verdictClass(v) {
    return ({
        strong_buy: 'sb-strong-buy',
        buy: 'sb-buy',
        hold: 'sb-hold',
        sell: 'sb-sell',
        strong_sell: 'sb-strong-sell',
    })[v] || '';
}
