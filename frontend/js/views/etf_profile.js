// ETF Profile + Holdings + Sector + Country — Finnhub ETF endpoints.
// Per-symbol drilldown: profile metadata, top holdings, sector mix, country mix.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderEtfProfile(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.etf_profile.h1.title">// ETF PROFILE</span></h1>
        <p class="muted small" data-i18n="view.etf_profile.hint.intro">
            Finnhub ETF endpoints — profile, top holdings, sector mix, country mix.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="etf-form">
                <label><span data-i18n="view.etf_profile.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="SPY" required></label>
                <button class="primary" type="submit" data-i18n="view.etf_profile.btn.load">Load</button>
            </form>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.etf_profile.h2.profile">Profile</h2>
                <div id="etf-profile"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.etf_profile.h2.holdings">Top holdings</h2>
                <div id="etf-holdings"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.etf_profile.h2.sectors">Sector mix</h2>
                <div id="etf-sectors"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.etf_profile.h2.countries">Country mix</h2>
                <div id="etf-countries"></div>
            </div>
        </div>
    `;
    document.getElementById('etf-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const els = ['etf-profile', 'etf-holdings', 'etf-sectors', 'etf-countries']
        .map(id => document.getElementById(id));
    els.forEach(el => el && (el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`));
    try {
        const [prof, hold, sec, ctry] = await Promise.all([
            api.finnhubEtfProfile(state.symbol).catch(() => null),
            api.finnhubEtfHoldings(state.symbol, 0).catch(() => null),
            api.finnhubEtfSector(state.symbol).catch(() => null),
            api.finnhubEtfCountry(state.symbol).catch(() => null),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderProfile(els[0], prof);
        renderHoldings(els[1], hold);
        renderMix(els[2], sec?.sectorExposure || [], 'sector');
        renderMix(els[3], ctry?.countryExposure || [], 'country');
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.etf_profile.toast.failed'), { level: 'error' });
    }
}

function renderProfile(el, p) {
    if (!el) return;
    const prof = p?.profile || p || {};
    if (!Object.keys(prof).length) {
        el.innerHTML = `<p class="muted" data-i18n="view.etf_profile.empty.profile">No profile.</p>`;
        return;
    }
    const rows = [
        [t('view.etf_profile.profile.name'),      prof.name],
        [t('view.etf_profile.profile.ticker'),    prof.ticker || prof.symbol],
        [t('view.etf_profile.profile.isin'),      prof.isin],
        [t('view.etf_profile.profile.aum'),       prof.aum ? '$' + prof.aum.toLocaleString() : null],
        [t('view.etf_profile.profile.expense_ratio'),
            prof.expenseRatio != null ? (prof.expenseRatio * 100).toFixed(2) + '%' : null],
        [t('view.etf_profile.profile.asset_class'),  prof.assetClass],
        [t('view.etf_profile.profile.investment_segment'), prof.investmentSegment],
        [t('view.etf_profile.profile.benchmark'),    prof.benchmark],
        [t('view.etf_profile.profile.issuer'),       prof.issuer],
        [t('view.etf_profile.profile.inception'),    prof.inceptionDate],
        [t('view.etf_profile.profile.website'),
            prof.website ? `<a href="${esc(prof.website)}" target="_blank">${esc(prof.website)}</a>` : null, true],
    ];
    el.innerHTML = `<table class="trades"><tbody>${rows
        .filter(([_, v]) => v != null && v !== '')
        .map(([k, v, html]) => `<tr><td>${k}</td><td>${html ? v : esc(String(v))}</td></tr>`)
        .join('')}</tbody></table>`;
}

function renderHoldings(el, h) {
    if (!el) return;
    const rows = h?.holdings || [];
    if (!rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.etf_profile.empty.holdings">No holdings.</p>`;
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.etf_profile.th.symbol">Symbol</th>
            <th data-i18n="view.etf_profile.th.name">Name</th>
            <th data-i18n="view.etf_profile.th.weight">Weight %</th>
            <th data-i18n="view.etf_profile.th.shares">Shares</th>
        </tr></thead>
        <tbody>${rows.slice(0, 50).map(r => `
            <tr>
                <td><a class="link" href="#research/${esc(r.symbol || '')}">${esc(r.symbol || '—')}</a></td>
                <td class="muted">${esc(r.name || '—')}</td>
                <td>${r.percent != null ? r.percent.toFixed(2) + '%' : '—'}</td>
                <td>${r.share != null ? r.share.toLocaleString() : '—'}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}

function renderMix(el, rows, kind) {
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<p class="muted">${esc(t(`view.etf_profile.empty.${kind}`))}</p>`;
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.etf_profile.th.label">Label</th>
            <th data-i18n="view.etf_profile.th.weight_2">Weight %</th>
        </tr></thead>
        <tbody>${rows.map(r => {
            const label = r.sector || r.country || r.exposure || '—';
            const w = (r.exposure ?? r.weight ?? r.percent ?? 0);
            return `<tr><td>${esc(label)}</td><td>${typeof w === 'number' ? w.toFixed(2) + '%' : '—'}</td></tr>`;
        }).join('')}</tbody>
    </table>`;
}
