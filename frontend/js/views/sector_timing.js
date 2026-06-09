// Sector RS rotation timing — ranks SPDR sectors by *entry-signal*
// strength rather than current daily leadership. Sectors with fresh
// MA crossovers + accelerating slope + breakout proximity outrank
// sectors already in steady leadership — that's where the
// rotation-trade alpha lives.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderSectorTiming(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sector_timing.title">// SECTOR RS ROTATION TIMING</span></h1>
        <p class="muted small" data-i18n-html="view.sector_timing.intro">
            Sits on top of the existing sector_rotation report (11 SPDR sectors vs SPY,
            60-day daily-RS sparkline per sector) and folds three timing signals into a
            per-sector composite score: (1) fresh 20d-MA-above-60d-MA crossover today,
            (2) 5-day OLS slope of cumulative RS exceeding the 20-day slope
            (acceleration), (3) current cumulative RS within 5% of — or above — its
            60-day high (breakout proximity). Ranks sectors <strong>entering</strong>
            leadership above sectors <strong>currently in</strong> steady leadership —
            that's where the rotation-trade alpha lives.
        </p>
        <div class="chart-panel">
            <div class="st-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm" id="st-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
                <span class="muted small" id="st-meta"></span>
            </div>
            <table class="trades" id="st-table">
                <thead><tr>
                    <th data-i18n="view.sector_timing.th.rank">#</th>
                    <th data-i18n="view.sector_timing.th.symbol">Sector</th>
                    <th data-i18n="view.sector_timing.th.score">Score</th>
                    <th data-i18n="view.sector_timing.th.crossover">Crossover</th>
                    <th data-i18n="view.sector_timing.th.ma">MA 20d / 60d</th>
                    <th data-i18n="view.sector_timing.th.accel">Accel</th>
                    <th data-i18n="view.sector_timing.th.slope">Slope 5d / 20d</th>
                    <th data-i18n="view.sector_timing.th.breakout">Breakout</th>
                    <th data-i18n="view.sector_timing.th.rs">RS / 60d-Hi</th>
                </tr></thead>
                <tbody><tr><td colspan="9" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#st-refresh').addEventListener('click', () => fetchAndRender(mount));
    fetchAndRender(mount);
}

async function fetchAndRender(mount) {
    const tbody = mount.querySelector('#st-table tbody');
    const meta = mount.querySelector('#st-meta');
    if (!tbody) return;
    tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('common.loading'))}</td></tr>`;
    try {
        const rows = await api('/sector-timing/ranked');
        if (!rows.length) {
            tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.sector_timing.empty.no_rows'))}</td></tr>`;
            return;
        }
        const enter = rows.filter(r => r.accelerating || r.fresh_bullish_crossover).length;
        if (meta) meta.textContent = t('view.sector_timing.meta.summary')
            .replace('{n}', rows.length).replace('{e}', enter);
        tbody.innerHTML = rows.map((r, i) => {
            const scoreCls = r.score >= 60 ? 'pos' : r.score >= 30 ? '' : 'muted';
            const crxCls = r.fresh_bullish_crossover ? 'pos' : r.ma_above ? '' : 'neg';
            const accCls = r.accelerating ? 'pos' : 'muted';
            const boCls = r.breakout_ratio >= 1.0 ? 'pos' : r.near_breakout ? '' : 'muted';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td class="muted">${i + 1}</td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong> <span class="muted small">${esc(r.label)}</span></td>
                <td class="${scoreCls}"><strong>${r.score.toFixed(1)}</strong></td>
                <td class="${crxCls}">${r.fresh_bullish_crossover ? 'FRESH ✓' : r.ma_above ? 'above' : 'below'}</td>
                <td class="muted small">${r.ma_short.toFixed(3)} / ${r.ma_long.toFixed(3)}</td>
                <td class="${accCls}">${r.accelerating ? '✓' : '—'}</td>
                <td class="muted small">${fmtSlope(r.slope_5d)} / ${fmtSlope(r.slope_20d)}</td>
                <td class="${boCls}">${r.breakout_ratio >= 1.0 ? 'BREAKOUT' : r.near_breakout ? 'near' : '—'}</td>
                <td class="muted small">${r.current_rs.toFixed(3)} / ${r.rs_60d_high.toFixed(3)}</td>
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(String(e))}</td></tr>`;
    }
}

function fmtSlope(n) {
    if (n == null) return '—';
    const bp = n * 10_000;
    return (bp >= 0 ? '+' : '') + bp.toFixed(1) + ' bp/d';
}
