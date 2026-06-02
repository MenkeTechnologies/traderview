// Price Target Consensus — Finnhub /stock/price-target + /quote.
// Computes implied upside vs current price.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderPriceTarget(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.price_target.h1.title">// PRICE TARGET CONSENSUS</span></h1>
        <p class="muted small" data-i18n="view.price_target.hint.intro">
            Wall Street analyst price targets: high / median / low + implied upside vs the
            current price. Fade trades: stock at PT-high with momentum dying = short setup.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="pt-form">
                <label><span data-i18n="view.price_target.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <button class="primary" type="submit" data-i18n="view.price_target.btn.load">Load</button>
            </form>
            <div id="pt-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('pt-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('pt-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const [pt, q] = await Promise.all([
            api.symbolPriceTarget(state.symbol),
            api.symbolFinnhubQuote(state.symbol).catch(() => null),
        ]);
        if (!viewIsCurrent(tok)) return;
        if (!pt || typeof pt !== 'object' || !pt.targetMean) {
            el.innerHTML = `<p class="muted" data-i18n="view.price_target.empty">No price target data.</p>`;
            return;
        }
        const price = Number(q?.c || 0);
        const upside = (target) => price > 0 && target ? ((target - price) / price * 100) : null;
        const median = Number(pt.targetMedian || pt.targetMean || 0);
        const high = Number(pt.targetHigh || 0);
        const low = Number(pt.targetLow || 0);
        const upsideMedian = upside(median);
        const cls = up => up == null ? '' : up >= 0 ? 'pos' : 'neg';
        el.innerHTML = `
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.price_target.card.current">Current</div>
                    <div class="value">${price ? '$' + price.toFixed(2) : '—'}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.price_target.card.median">Median PT</div>
                    <div class="value">${median ? '$' + median.toFixed(2) : '—'}</div>
                </div>
                <div class="card ${cls(upsideMedian)}">
                    <div class="label" data-i18n="view.price_target.card.upside_median">Upside (median)</div>
                    <div class="value">${upsideMedian != null ? (upsideMedian >= 0 ? '+' : '') + upsideMedian.toFixed(1) + '%' : '—'}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.price_target.card.high">High PT</div>
                    <div class="value">${high ? '$' + high.toFixed(2) : '—'} <span class="muted small">${upside(high) != null ? (upside(high) >= 0 ? '+' : '') + upside(high).toFixed(0) + '%' : ''}</span></div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.price_target.card.low">Low PT</div>
                    <div class="value">${low ? '$' + low.toFixed(2) : '—'} <span class="muted small">${upside(low) != null ? (upside(low) >= 0 ? '+' : '') + upside(low).toFixed(0) + '%' : ''}</span></div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.price_target.card.analysts">Analysts</div>
                    <div class="value">${pt.numberAnalysts ?? '—'}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.price_target.card.updated">Last updated</div>
                    <div class="value">${esc(pt.lastUpdated || '—')}</div>
                </div>
            </div>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.price_target.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.price_target.toast.failed'), { level: 'error' });
    }
}
