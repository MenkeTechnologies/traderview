// Market gamma regime tracker — portfolio-level vol-regime indicator.
// Positive total dealer GEX → vol-suppressing, mean-reverting tape
// (sell premium, fade extremes). Negative → vol-amplifying, momentum
// tape (long vol, trend follow). Flip moments bracket the biggest
// SPX vol expansions historically.

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';

export async function renderMarketGammaRegime(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.market_gamma_regime.title">// MARKET GAMMA REGIME · SPY DEALER GEX</span></h1>
        <p class="muted small" data-i18n-html="view.market_gamma_regime.intro">
            Computes total SPY-level dealer GEX across the nearest 4 option expirations
            every 30 min via the existing <code>gex_scanner</code> + Black-Scholes
            gamma pipeline. <strong>Positive GEX</strong>: dealers long gamma →
            sell-into-rallies / buy-into-dips re-hedging → vol-suppressing,
            mean-reverting tape. <strong>Negative GEX</strong>: dealers short gamma
            → buy-rallies / sell-dips re-hedging → vol-amplifying, momentum tape.
            <strong>Flip moments</strong> bracket the biggest historical SPX vol
            expansions (Squeeze Metrics, JPM Equity Derivatives, Goldman quant desk
            research). Portfolio-level regime indicator — not a per-trade signal —
            it tells you which strategies have a tailwind vs headwind right now.
        </p>
        <div class="chart-panel">
            <div class="mg-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm primary" id="mg-refresh" data-shortcut="r" data-i18n="view.market_gamma_regime.btn.refresh_now">⚡ Refresh Now</button>
                <span class="muted small" id="mg-meta"></span>
            </div>
            <div id="mg-current" class="mg-current-panel"></div>
            <h2 style="margin-top:1rem" data-i18n="view.market_gamma_regime.h2.flips">Recent Regime Flips</h2>
            <table class="trades" id="mg-flips">
                <thead><tr>
                    <th data-i18n="view.market_gamma_regime.th.when">When</th>
                    <th data-i18n="view.market_gamma_regime.th.from">From</th>
                    <th data-i18n="view.market_gamma_regime.th.to">To</th>
                    <th data-i18n="view.market_gamma_regime.th.prior_gex">Prior GEX</th>
                    <th data-i18n="view.market_gamma_regime.th.current_gex">Post-Flip GEX</th>
                </tr></thead>
                <tbody><tr><td colspan="5" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
            <h2 style="margin-top:1rem" data-i18n="view.market_gamma_regime.h2.history">GEX History</h2>
            <table class="trades" id="mg-history">
                <thead><tr>
                    <th data-i18n="view.market_gamma_regime.th.observed">Observed</th>
                    <th data-i18n="view.market_gamma_regime.th.regime">Regime</th>
                    <th data-i18n="view.market_gamma_regime.th.total_gex">Total GEX</th>
                    <th data-i18n="view.market_gamma_regime.th.spot">Spot</th>
                    <th data-i18n="view.market_gamma_regime.th.expirations">Expirations</th>
                </tr></thead>
                <tbody><tr><td colspan="5" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#mg-refresh').addEventListener('click', async () => {
        const meta = mount.querySelector('#mg-meta');
        if (meta) meta.textContent = t('view.market_gamma_regime.status.refreshing');
        // Surface refresh failures — the silent catch hid the 405 from
        // the GET/POST route mismatch and the button read as a no-op.
        try { await api.marketGammaRefresh(); }
        catch (e) { showToast(t('toast.error.api', { err: e.message }), { level: 'error' }); }
        fetchAndRender(mount);
    });
    fetchAndRender(mount);
}

async function fetchAndRender(mount) {
    const cur = mount.querySelector('#mg-current');
    const flipsTbody = mount.querySelector('#mg-flips tbody');
    const histTbody = mount.querySelector('#mg-history tbody');
    const meta = mount.querySelector('#mg-meta');
    try {
        const r = await api.marketGammaReport();
        if (!r) {
            cur.innerHTML = `<p class="muted">${esc(t('view.market_gamma_regime.empty.no_data'))}</p>`;
            flipsTbody.innerHTML = '';
            histTbody.innerHTML = '';
            if (meta) meta.textContent = '';
            return;
        }
        const regCls = r.current_regime === 'positive' ? 'pos' : r.current_regime === 'negative' ? 'neg' : 'muted';
        if (meta) meta.textContent = t('view.market_gamma_regime.meta.summary')
            .replace('{n}', r.history.length).replace('{f}', r.recent_flips.length);
        cur.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px">
                <div><div class="muted small">${esc(t('view.market_gamma_regime.field.current_regime'))}</div>
                     <strong class="${regCls}" style="font-size:1.5em">${esc(r.current_regime.toUpperCase())}</strong></div>
                <div><div class="muted small">${esc(t('view.market_gamma_regime.field.total_gex'))}</div>
                     <strong>${fmtGex(r.current_total_gex_usd)}</strong></div>
                <div><div class="muted small">${esc(t('view.market_gamma_regime.field.spot'))}</div>
                     <strong>${r.current_spot.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.market_gamma_regime.field.time_in_regime'))}</div>
                     <strong>${fmtDuration(r.time_in_regime_secs)}</strong></div>
                <div><div class="muted small">${esc(t('view.market_gamma_regime.field.last_observed'))}</div>
                     <strong class="muted small">${esc(fmtDateTime(r.last_observed_at))}</strong></div>
            </div>
        `;
        if (!r.recent_flips.length) {
            flipsTbody.innerHTML = `<tr><td colspan="5" class="muted">${esc(t('view.market_gamma_regime.empty.no_flips'))}</td></tr>`;
        } else {
            flipsTbody.innerHTML = r.recent_flips.slice().reverse().map(f => `
                <tr>
                    <td>${esc(fmtDateTime(f.flipped_at))}</td>
                    <td class="${regimeCls(f.from)}">${esc(f.from)}</td>
                    <td class="${regimeCls(f.to)}"><strong>${esc(f.to)}</strong></td>
                    <td>${fmtGex(f.prior_total_gex_usd)}</td>
                    <td>${fmtGex(f.current_total_gex_usd)}</td>
                </tr>
            `).join('');
        }
        if (!r.history.length) {
            histTbody.innerHTML = `<tr><td colspan="5" class="muted">${esc(t('view.market_gamma_regime.empty.no_history'))}</td></tr>`;
        } else {
            const slice = r.history.slice(-50).reverse();
            histTbody.innerHTML = slice.map(s => `
                <tr>
                    <td>${esc(fmtDateTime(s.observed_at))}</td>
                    <td class="${regimeCls(s.regime)}">${esc(s.regime)}</td>
                    <td>${fmtGex(s.total_gex_usd)}</td>
                    <td>${s.spot.toFixed(2)}</td>
                    <td class="muted small">${(s.expirations_used || []).map(esc).join(', ')}</td>
                </tr>
            `).join('');
        }
    } catch (e) {
        cur.innerHTML = `<p class="muted">${esc(String(e))}</p>`;
    }
}

function regimeCls(r) {
    return r === 'positive' ? 'pos' : r === 'negative' ? 'neg' : 'muted';
}

function fmtGex(n) {
    if (n == null || !Number.isFinite(n)) return '—';
    const abs = Math.abs(n);
    const sign = n < 0 ? '-' : '+';
    if (abs >= 1_000_000_000) return `${sign}$${(abs / 1_000_000_000).toFixed(2)}B`;
    if (abs >= 1_000_000) return `${sign}$${(abs / 1_000_000).toFixed(0)}M`;
    return `${sign}$${abs.toFixed(0)}`;
}

function fmtDuration(secs) {
    if (secs == null) return '—';
    if (secs < 60) return secs + 's';
    if (secs < 3600) return Math.round(secs / 60) + 'm';
    if (secs < 86400) return (secs / 3600).toFixed(1) + 'h';
    return (secs / 86400).toFixed(1) + 'd';
}
