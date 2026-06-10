// Robert Shiller's CAPE (Cyclically-Adjusted Price-to-Earnings) ratio
// indicator. CAPE = S&P 500 price / 10-year inflation-adjusted average
// earnings. Shiller's regression: high CAPE → low forward 10y returns.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderCapeIndicator(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cape_indicator.title">// CAPE · SHILLER P/E</span></h1>
        <p class="muted small" data-i18n-html="view.cape_indicator.intro">
            Robert Shiller's <strong>Cyclically-Adjusted P/E ratio</strong> = S&P 500
            price ÷ 10-year inflation-adjusted average earnings. Shiller's seminal
            work (<em>Irrational Exuberance</em>, 2000) showed CAPE correlates with
            subsequent 10-year returns: high CAPE → low future returns, low CAPE →
            high future returns. Historical distribution (1881-2024): mean ~17,
            median ~16, max 44.2 (Dec 1999 dot-com peak).
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label>
                    <span class="muted small" data-i18n="view.cape_indicator.field.value">Current CAPE</span>
                    <input type="number" id="cape-value" step="0.5" min="1" max="100" placeholder="(leave blank for latest known)" style="width:160px">
                </label>
                <button class="btn btn-sm primary" id="cape-run" data-shortcut="r" data-i18n="view.cape_indicator.btn.score">⚡ Score</button>
                <span class="muted small" id="cape-meta"></span>
            </div>
            <div id="cape-result"></div>
        </div>
    `;
    mount.querySelector('#cape-run').addEventListener('click', () => runScore(mount));
    await runScore(mount);
}

async function runScore(mount) {
    const result = mount.querySelector('#cape-result');
    const meta = mount.querySelector('#cape-meta');
    const val = mount.querySelector('#cape-value').value.trim();
    const url = val ? `/cape-indicator/score?value=${val}` : '/cape-indicator/score';
    result.innerHTML = `<p class="muted">${esc(t('view.cape_indicator.status.scoring'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api.request(url);
        const regimeCls = {
            depressed: 'pos',
            below_avg: 'pos',
            near_avg: '',
            elevated: 'neg',
            extreme: 'neg',
        }[r.regime] || '';
        if (meta) meta.textContent = t('view.cape_indicator.meta.summary').replace('{p}', r.percentile_pct.toFixed(1));
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-bottom:12px">
                <div><div class="muted small">${esc(t('view.cape_indicator.field.current'))}</div>
                    <strong style="font-size:1.4em">${r.current_value.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.cape_indicator.field.regime'))}</div>
                    <strong class="${regimeCls}">${esc(r.regime.toUpperCase().replace('_', ' '))}</strong></div>
                <div><div class="muted small">${esc(t('view.cape_indicator.field.percentile'))}</div>
                    <strong class="${regimeCls}">${r.percentile_pct.toFixed(1)}%</strong></div>
                <div><div class="muted small">${esc(t('view.cape_indicator.field.historical_mean'))}</div>
                    <strong>${r.historical_mean.toFixed(1)}</strong></div>
                <div><div class="muted small">${esc(t('view.cape_indicator.field.historical_median'))}</div>
                    <strong>${r.historical_median.toFixed(1)}</strong></div>
                <div><div class="muted small">${esc(t('view.cape_indicator.field.historical_max'))}</div>
                    <strong>${r.historical_max.toFixed(1)}</strong></div>
            </div>
            <p class="${regimeCls}"><strong>${esc(t('view.cape_indicator.field.interpretation'))}:</strong> ${esc(r.interpretation)}</p>
            <h2 style="margin-top:1rem">${esc(t('view.cape_indicator.h2.recent'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.cape_indicator.th.period">Period</th>
                    <th data-i18n="view.cape_indicator.th.value">CAPE</th>
                </tr></thead>
                <tbody>${(r.recent_quarterly || []).slice().reverse().map(q => `
                    <tr>
                        <td class="muted small">Q${q.quarter} ${q.year}</td>
                        <td><strong>${q.value.toFixed(1)}</strong></td>
                    </tr>
                `).join('')}</tbody>
            </table>
            <p class="muted small">${esc(t('view.cape_indicator.hint.source'))}</p>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
