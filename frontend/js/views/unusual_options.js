// Unusual Options Activity — Finnhub /stock/option-chain.
// Detects strikes with vol > 3 × OI (i.e. someone just opened a big position).
// Premium endpoint — graceful empty on free tier.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '', minVolOiRatio: 3.0, minVolume: 500 };

export async function renderUnusualOptions(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.uoa.h1.title">// UNUSUAL OPTIONS ACTIVITY</span></h1>
        <p class="muted small" data-i18n="view.uoa.hint.intro">
            Strikes where volume &gt; N × open interest. Indicates fresh positioning
            (someone just opened a big bet vs. existing book). Smart-money tracking signal.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="uoa-form">
                <label><span data-i18n="view.uoa.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="NVDA" required></label>
                <label><span data-i18n="view.uoa.label.vol_oi">Vol/OI threshold</span>
                    <input type="number" step="0.1" name="ratio" value="${state.minVolOiRatio}" min="1"></label>
                <label><span data-i18n="view.uoa.label.min_vol">Min volume</span>
                    <input type="number" step="0.01" name="min_vol" value="${state.minVolume}" min="1"></label>
                <button class="primary" type="submit" data-i18n="view.uoa.btn.scan">Scan</button>
            </form>
            <div id="uoa-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('uoa-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        state.minVolOiRatio = Number(fd.get('ratio')) || 3.0;
        state.minVolume = Number(fd.get('min_vol')) || 500;
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('uoa-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const data = await api.symbolOptionChain(state.symbol);
        if (!viewIsCurrent(tok)) return;
        const chains = data?.data || [];
        if (!chains.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.uoa.empty">No option chain (or premium required).</p>`;
            return;
        }
        // Flatten + filter every expiry × strike.
        const unusual = [];
        for (const exp of chains) {
            for (const opt of (exp.options?.CALL || [])) {
                pushIfUnusual(unusual, opt, exp.expirationDate, 'call');
            }
            for (const opt of (exp.options?.PUT || [])) {
                pushIfUnusual(unusual, opt, exp.expirationDate, 'put');
            }
        }
        if (!unusual.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.uoa.empty_filtered">No strikes pass current thresholds. Loosen them or try a different symbol.</p>`;
            return;
        }
        unusual.sort((a, b) => b.ratio - a.ratio);
        const totalCallVol = unusual.filter(u => u.side === 'call').reduce((s, u) => s + u.volume, 0);
        const totalPutVol = unusual.filter(u => u.side === 'put').reduce((s, u) => s + u.volume, 0);
        const pcr = totalCallVol > 0 ? (totalPutVol / totalCallVol) : 0;
        el.innerHTML = `
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.uoa.card.strikes">Unusual strikes</div>
                    <div class="value">${unusual.length}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.uoa.card.call_vol">Call vol</div>
                    <div class="value">${totalCallVol.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.uoa.card.put_vol">Put vol</div>
                    <div class="value">${totalPutVol.toLocaleString()}</div></div>
                <div class="card ${pcr > 1 ? 'neg' : pcr < 0.7 ? 'pos' : ''}">
                    <div class="label" data-i18n="view.uoa.card.pcr">Put/Call ratio</div>
                    <div class="value">${pcr.toFixed(2)}</div></div>
            </div>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.uoa.th.expiry">Expiry</th>
                    <th data-i18n="view.uoa.th.strike">Strike</th>
                    <th data-i18n="view.uoa.th.side">Side</th>
                    <th data-i18n="view.uoa.th.volume">Volume</th>
                    <th data-i18n="view.uoa.th.oi">OI</th>
                    <th data-i18n="view.uoa.th.ratio">Vol/OI</th>
                    <th data-i18n="view.uoa.th.last">Last</th>
                    <th data-i18n="view.uoa.th.iv">IV</th>
                    <th data-i18n="view.uoa.th.delta">Δ</th>
                </tr></thead>
                <tbody>${unusual.slice(0, 200).map(u => `
                    <tr>
                        <td>${esc(u.expiry)}</td>
                        <td>${u.strike.toFixed(2)}</td>
                        <td class="${u.side === 'call' ? 'pos' : 'neg'}">${u.side.toUpperCase()}</td>
                        <td>${u.volume.toLocaleString()}</td>
                        <td class="muted">${u.openInterest.toLocaleString()}</td>
                        <td class="pos">${u.ratio.toFixed(1)}×</td>
                        <td>${u.last.toFixed(2)}</td>
                        <td class="muted">${u.iv != null ? (u.iv * 100).toFixed(1) + '%' : '—'}</td>
                        <td class="muted">${u.delta != null ? u.delta.toFixed(2) : '—'}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.uoa.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.uoa.toast.failed'), { level: 'error' });
    }
}

function pushIfUnusual(arr, opt, expiry, side) {
    const vol = Number(opt.volume || 0);
    const oi = Number(opt.openInterest || 0);
    if (vol < state.minVolume) return;
    if (oi <= 0) return;
    const ratio = vol / oi;
    if (ratio < state.minVolOiRatio) return;
    arr.push({
        expiry,
        side,
        strike: Number(opt.strike || 0),
        volume: vol,
        openInterest: oi,
        ratio,
        last: Number(opt.lastPrice || 0),
        iv: opt.impliedVolatility != null ? Number(opt.impliedVolatility) : null,
        delta: opt.delta != null ? Number(opt.delta) : null,
    });
}
