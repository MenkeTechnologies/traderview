// Mutual Fund Profile + Holdings — Finnhub /mutual-fund/profile + /holdings
// + /sector + /country. Combined deep-dive on any open-end fund.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderMutualFund(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mf.h1.title">// MUTUAL FUND PROFILE</span></h1>
        <p class="muted small" data-i18n="view.mf.hint.intro">
            Finnhub mutual fund endpoints — profile, top holdings, sector mix, country mix.
            Compare expense ratios + tilt vs your own holdings.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="mf-form">
                <label><span data-i18n="view.mf.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="FXAIX / VTIAX" required></label>
                <button class="primary" type="submit" data-i18n="view.mf.btn.load">Load</button>
            </form>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.mf.h2.profile">Profile</h2>
                <div id="mf-profile"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.mf.h2.holdings">Top holdings</h2>
                <div id="mf-holdings"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.mf.h2.sectors">Sector mix</h2>
                <div id="mf-sectors"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.mf.h2.countries">Country mix</h2>
                <div id="mf-countries"></div>
            </div>
        </div>
    `;
    document.getElementById('mf-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const els = ['mf-profile', 'mf-holdings', 'mf-sectors', 'mf-countries']
        .map(id => document.getElementById(id));
    els.forEach(el => el && (el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`));
    try {
        const [prof, hold, sec, ctry] = await Promise.all([
            api.finnhubMfProfile(state.symbol).catch(() => null),
            api.finnhubMfHoldings(state.symbol, 0).catch(() => null),
            api.finnhubMfSector(state.symbol).catch(() => null),
            api.finnhubMfCountry(state.symbol).catch(() => null),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderProfile(els[0], prof);
        renderHoldings(els[1], hold);
        renderMix(els[2], sec?.sectorExposure || sec?.exposure || [], 'sector');
        renderMix(els[3], ctry?.countryExposure || ctry?.exposure || [], 'country');
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.mf.toast.failed'), { level: 'error' });
    }
}

function renderProfile(el, p) {
    if (!el) return;
    const prof = p?.profile || p || {};
    if (!Object.keys(prof).length) {
        el.innerHTML = `<p class="muted" data-i18n="view.mf.empty.profile">No profile (or premium required).</p>`;
        return;
    }
    const rows = [
        [t('view.mf.profile.name'),       prof.name],
        [t('view.mf.profile.category'),   prof.category],
        [t('view.mf.profile.isin'),       prof.isin],
        [t('view.mf.profile.nav'),        prof.nav ? '$' + Number(prof.nav).toFixed(2) : null],
        [t('view.mf.profile.aum'),        prof.totalNetAssets ? '$' + Number(prof.totalNetAssets).toLocaleString() : null],
        [t('view.mf.profile.expense'),    prof.expenseRatio != null ? (Number(prof.expenseRatio) * 100).toFixed(2) + '%' : null],
        [t('view.mf.profile.benchmark'),  prof.benchmark],
        [t('view.mf.profile.fund_family'), prof.fundFamily],
        [t('view.mf.profile.inception'),  prof.inceptionDate],
        [t('view.mf.profile.manager'),    prof.manager],
        [t('view.mf.profile.min_invest'), prof.minInvestment ? '$' + Number(prof.minInvestment).toLocaleString() : null],
    ];
    el.innerHTML = `<table class="trades"><tbody>${rows
        .filter(([_, v]) => v != null && v !== '')
        .map(([k, v]) => `<tr><td>${k}</td><td>${esc(String(v))}</td></tr>`)
        .join('')}</tbody></table>`;
}

function renderHoldings(el, h) {
    if (!el) return;
    const rows = h?.holdings || [];
    if (!rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.mf.empty.holdings">No holdings.</p>`;
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.mf.th.symbol">Symbol</th>
            <th data-i18n="view.mf.th.name">Name</th>
            <th data-i18n="view.mf.th.weight">Weight %</th>
            <th data-i18n="view.mf.th.shares">Shares</th>
        </tr></thead>
        <tbody>${rows.slice(0, 50).map(r => `
            <tr>
                <td><a class="link" href="#research/${esc(r.symbol || '')}">${esc(r.symbol || '—')}</a></td>
                <td class="muted">${esc(r.name || '—')}</td>
                <td>${r.percent != null ? Number(r.percent).toFixed(2) + '%' : '—'}</td>
                <td>${r.share != null ? Number(r.share).toLocaleString() : '—'}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}

function renderMix(el, rows, kind) {
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<p class="muted">${esc(t(`view.mf.empty.${kind}`))}</p>`;
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.mf.th.label">Label</th>
            <th data-i18n="view.mf.th.weight_2">Weight %</th>
        </tr></thead>
        <tbody>${rows.map(r => {
            const label = r.sector || r.country || r.exposure || '—';
            const w = (r.exposure ?? r.weight ?? r.percent ?? 0);
            return `<tr><td>${esc(label)}</td><td>${typeof w === 'number' ? w.toFixed(2) + '%' : '—'}</td></tr>`;
        }).join('')}</tbody>
    </table>`;
}
