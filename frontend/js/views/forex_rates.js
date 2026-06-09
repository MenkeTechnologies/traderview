// Forex Rates — Finnhub /forex/rates consumer. Real-time spot rates for
// a base currency against every supported quote currency.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const BASES = ['USD', 'EUR', 'GBP', 'JPY', 'CHF', 'CAD', 'AUD', 'NZD', 'CNY', 'INR'];
let state = { base: 'USD' };

export async function renderForexRates(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.forex_rates.h1.title">// FOREX RATES</span></h1>
        <p class="muted small" data-i18n="view.forex_rates.hint.intro">
            Real-time forex rates from Finnhub. Pick a base currency; the table shows
            how many units of each quote currency 1 unit of the base buys.
        </p>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.forex_rates.label.base">Base</span>
                    <select id="fx-base">${BASES.map(b =>
                        `<option value="${b}" ${b === state.base ? 'selected' : ''}>${b}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" id="fx-refresh" type="button" data-i18n="view.forex_rates.btn.refresh">Refresh</button>
            </div>
            <div id="fx-table" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('fx-base').addEventListener('change', e => {
        state.base = e.target.value;
        void load(tok);
    });
    document.getElementById('fx-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

async function load(tok) {
    const el = document.getElementById('fx-table');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.finnhubForexRates(state.base);
        if (!viewIsCurrent(tok)) return;
        const quote = data?.quote || {};
        const rows = Object.entries(quote)
            .map(([k, v]) => ({ ccy: k, rate: Number(v) }))
            .filter(r => Number.isFinite(r.rate))
            .sort((a, b) => a.ccy.localeCompare(b.ccy));
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.forex_rates.empty">No rate data for this base.</p>`;
            return;
        }
        el.innerHTML = `<table class="trades">
            <thead><tr>
                <th data-i18n="view.forex_rates.th.quote_ccy">Quote ccy</th>
                <th data-i18n="view.forex_rates.th.rate">Rate</th>
            </tr></thead>
            <tbody>${rows.map(r =>
                `<tr><td>${esc(r.ccy)}</td><td>${r.rate.toFixed(6)}</td></tr>`
            ).join('')}</tbody>
        </table>`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.forex_rates.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.forex_rates.toast.failed'), { level: 'error' });
    }
}
