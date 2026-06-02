// Crypto Markets — exchanges + symbols per exchange + per-coin profile.
// Uses Finnhub /crypto/exchange + /crypto/symbol + /crypto/profile.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { exchange: 'binance', profileQuery: '' };

export async function renderCryptoMarkets(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.crypto_markets.h1.title">// CRYPTO MARKETS</span></h1>
        <p class="muted small" data-i18n="view.crypto_markets.hint.intro">
            Finnhub crypto endpoints — supported exchanges, traded pairs per exchange,
            per-coin fundamentals (founding year, market cap rank, max supply).
        </p>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.crypto_markets.h2.exchanges">Exchanges</h2>
                <div id="cm-exchanges"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.crypto_markets.h2.symbols">Symbols on exchange</h2>
                <div class="inline-form">
                    <label><span data-i18n="view.crypto_markets.label.exchange">Exchange</span>
                        <input type="text" id="cm-exchange-input" value="${esc(state.exchange)}"></label>
                    <button id="cm-load-symbols" class="primary" type="button" data-i18n="view.crypto_markets.btn.load">Load</button>
                </div>
                <div id="cm-symbols" style="margin-top:10px"></div>
            </div>
            <div class="chart-panel" style="grid-column:1/-1">
                <h2 data-i18n="view.crypto_markets.h2.profile">Coin profile</h2>
                <div class="inline-form">
                    <label><span data-i18n="view.crypto_markets.label.symbol">Symbol (e.g. BINANCE:BTCUSDT)</span>
                        <input type="text" id="cm-profile-input" value="${esc(state.profileQuery)}" style="min-width:280px"></label>
                    <button id="cm-load-profile" class="primary" type="button" data-i18n="view.crypto_markets.btn.load_profile">Load profile</button>
                </div>
                <div id="cm-profile" style="margin-top:10px"></div>
            </div>
        </div>
    `;
    document.getElementById('cm-load-symbols').addEventListener('click', () => {
        state.exchange = document.getElementById('cm-exchange-input').value.trim() || 'binance';
        void loadSymbols(tok);
    });
    document.getElementById('cm-load-profile').addEventListener('click', () => {
        state.profileQuery = document.getElementById('cm-profile-input').value.trim();
        void loadProfile(tok);
    });
    await Promise.all([loadExchanges(tok), loadSymbols(tok)]);
}

async function loadExchanges(tok) {
    const el = document.getElementById('cm-exchanges');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const data = await api.finnhubCryptoExchanges();
        if (!viewIsCurrent(tok)) return;
        const list = Array.isArray(data) ? data : [];
        if (!list.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.crypto_markets.empty.exchanges">No exchanges.</p>`;
            return;
        }
        el.innerHTML = list.map(x =>
            `<span class="tile-badge" style="margin:2px;display:inline-block;cursor:pointer" data-ex="${esc(x)}">${esc(x)}</span>`
        ).join('');
        el.querySelectorAll('[data-ex]').forEach(badge => {
            badge.addEventListener('click', () => {
                document.getElementById('cm-exchange-input').value = badge.dataset.ex;
                state.exchange = badge.dataset.ex;
                void loadSymbols(tok);
            });
        });
    } catch (e) {
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.crypto_markets.error.exchanges', { msg: e.message || e }))}</p>`;
    }
}

async function loadSymbols(tok) {
    const el = document.getElementById('cm-symbols');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const list = await api.finnhubCryptoSymbols(state.exchange);
        if (!viewIsCurrent(tok)) return;
        const rows = Array.isArray(list) ? list : [];
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.crypto_markets.empty.symbols">No symbols.</p>`;
            return;
        }
        el.innerHTML = `
            <p class="muted small">
                <span data-i18n="view.crypto_markets.label.count">Pairs:</span>
                <strong>${rows.length}</strong>
            </p>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.crypto_markets.th.symbol">Symbol</th>
                    <th data-i18n="view.crypto_markets.th.display">Display</th>
                    <th data-i18n="view.crypto_markets.th.description">Description</th>
                </tr></thead>
                <tbody>${rows.slice(0, 200).map(r => `
                    <tr>
                        <td><code>${esc(r.symbol || '—')}</code></td>
                        <td>${esc(r.displaySymbol || '—')}</td>
                        <td class="muted">${esc(r.description || '—')}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.crypto_markets.error.symbols', { msg: e.message || e }))}</p>`;
    }
}

async function loadProfile(tok) {
    if (!state.profileQuery) return;
    const el = document.getElementById('cm-profile');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const p = await api.finnhubCryptoProfile(state.profileQuery);
        if (!viewIsCurrent(tok)) return;
        if (!p || typeof p !== 'object' || !Object.keys(p).length) {
            el.innerHTML = `<p class="muted" data-i18n="view.crypto_markets.empty.profile">No profile.</p>`;
            return;
        }
        const rows = [
            [t('view.crypto_markets.profile.name'),       p.name],
            [t('view.crypto_markets.profile.symbol'),     p.symbol],
            [t('view.crypto_markets.profile.long_name'),  p.longName],
            [t('view.crypto_markets.profile.market_cap_rank'), p.marketCapRank],
            [t('view.crypto_markets.profile.launch_date'), p.launchDate],
            [t('view.crypto_markets.profile.proof_type'),  p.proofType],
            [t('view.crypto_markets.profile.max_supply'),
                p.maxSupply != null ? Number(p.maxSupply).toLocaleString() : null],
            [t('view.crypto_markets.profile.circulating_supply'),
                p.circulatingSupply != null ? Number(p.circulatingSupply).toLocaleString() : null],
            [t('view.crypto_markets.profile.algorithm'),   p.algorithm],
        ];
        el.innerHTML = `<table class="trades"><tbody>${rows
            .filter(([_, v]) => v != null && v !== '')
            .map(([k, v]) => `<tr><td>${k}</td><td>${esc(String(v))}</td></tr>`)
            .join('')}</tbody></table>`;
        if (p.description) {
            el.insertAdjacentHTML('beforeend',
                `<details style="margin-top:8px"><summary data-i18n="view.crypto_markets.summary.description">Description</summary>
                 <p class="muted small">${esc(p.description.slice(0, 800))}${p.description.length > 800 ? '…' : ''}</p>
                 </details>`);
        }
    } catch (e) {
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.crypto_markets.error.profile', { msg: e.message || e }))}</p>`;
        showToast(t('view.crypto_markets.toast.failed'), { level: 'error' });
    }
}
