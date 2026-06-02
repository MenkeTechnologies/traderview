// Finnhub Aggregate Indicator — composite buy/sell/neutral signal across
// multiple technical indicators. Per-symbol via /scan/technical-indicator.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const RESOLUTIONS = ['15', '30', '60', 'D', 'W', 'M'];
let state = { symbol: '', resolution: 'D' };

export async function renderFinnhubAggregate(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.finnhub_aggregate.h1.title">// AGGREGATE TECHNICAL SIGNAL</span></h1>
        <p class="muted small" data-i18n="view.finnhub_aggregate.hint.intro">
            Composite signal from MA + ADX + RSI + Stoch + MACD + CCI. Score ∈ [-N, +N];
            classification: BUY / NEUTRAL / SELL. Useful as a sanity check on your own technicals.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="fa-form">
                <label><span data-i18n="view.finnhub_aggregate.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <label><span data-i18n="view.finnhub_aggregate.label.resolution">Resolution</span>
                    <select name="resolution">${RESOLUTIONS.map(r =>
                        `<option value="${r}" ${r === state.resolution ? 'selected' : ''}>${r}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" type="submit" data-i18n="view.finnhub_aggregate.btn.scan">Compute</button>
            </form>
            <div id="fa-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('fa-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        state.resolution = fd.get('resolution') || 'D';
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('fa-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const data = await api.symbolScanAggregate(state.symbol, state.resolution);
        if (!viewIsCurrent(tok)) return;
        if (!data || typeof data !== 'object') {
            el.innerHTML = `<p class="muted" data-i18n="view.finnhub_aggregate.empty">No signal data.</p>`;
            return;
        }
        const tech = data.technicalAnalysis || {};
        const trend = data.trend || {};
        const sig = (tech.signal || '').toLowerCase();
        const cls = sig === 'buy' ? 'pos' : sig === 'sell' ? 'neg' : '';
        const adxLabel = trend.adx
            ? (trend.adx >= 25 ? t('view.finnhub_aggregate.label.strong_trend')
                               : t('view.finnhub_aggregate.label.weak_trend'))
            : '—';
        el.innerHTML = `
            <div class="cards">
                <div class="card ${cls}">
                    <div class="label" data-i18n="view.finnhub_aggregate.label.signal">Signal</div>
                    <div class="value">${esc((tech.signal || '—').toUpperCase())}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.finnhub_aggregate.label.count">Indicator votes</div>
                    <div class="value">
                        <span class="pos">${tech.count?.buy ?? 0} buy</span>
                        · ${tech.count?.neutral ?? 0} neut
                        · <span class="neg">${tech.count?.sell ?? 0} sell</span>
                    </div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.finnhub_aggregate.label.adx">ADX</div>
                    <div class="value">${trend.adx?.toFixed(1) ?? '—'} <span class="muted small">${esc(adxLabel)}</span></div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.finnhub_aggregate.label.trending">Trending direction</div>
                    <div class="value">${trend.trending === true ? '↑' : trend.trending === false ? '↓' : '—'}</div>
                </div>
            </div>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.finnhub_aggregate.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.finnhub_aggregate.toast.failed'), { level: 'error' });
    }
}
