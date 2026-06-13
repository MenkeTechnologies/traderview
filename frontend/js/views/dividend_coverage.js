// Dividend coverage & payout — payout ratio, earnings coverage, retention,
// optional FCF payout, and a sustainability rating, via
// /calc/dividend-coverage. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['eps_usd', 'Earnings per share ($)', 5],
    ['dps_usd', 'Dividends per share ($)', 2],
    ['fcf_per_share_usd', 'FCF per share ($, 0 = skip)', 8],
];

const pct = (n) => Number(n).toFixed(1) + '%';
const RATING = {
    healthy: ['Healthy (<60%)', 'pos'],
    moderate: ['Moderate (60–90%)', ''],
    stretched: ['Stretched (90–100%)', 'neg'],
    unsustainable: ['Unsustainable (>100%)', 'neg'],
    'no dividend': ['No dividend', ''],
};

export async function renderDividendCoverage(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.divc.h1.title">// DIVIDEND COVERAGE</span></h1>
        <p class="muted small" data-i18n="view.divc.hint.intro">
            Is a stock's dividend sustainable? The payout ratio (dividends ÷ EPS) is the share
            of earnings paid out — over 100% means the dividend exceeds earnings, funded from
            cash or debt. Coverage (EPS ÷ DPS) is how many times earnings cover it; retention is
            what's plowed back. The FCF payout (dividends ÷ free cash flow) is a stricter test,
            since dividends are paid from cash, not accounting earnings. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.divc.h2.inputs">Per share</h2>
            <form id="divc-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.divc.label.${key}">${label}</span>
                        <input type="number" step="0.01" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="divc-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#divc-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcDividendCoverage(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.divc.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#divc-result');
    const [ratingLabel, ratingCls] = RATING[r.rating] || [r.rating, ''];
    const coverage = r.coverage_ratio == null ? '—' : Number(r.coverage_ratio).toFixed(2) + '×';
    const fcf = r.fcf_payout_pct == null ? '—' : pct(r.fcf_payout_pct);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.divc.h2.result">The verdict</h2>
            <div class="cards">
                <div class="card ${ratingCls}"><div class="label" data-i18n="view.divc.card.payout">Payout ratio</div>
                    <div class="value ${ratingCls}">${pct(r.payout_ratio_pct)}</div></div>
                <div class="card ${ratingCls}"><div class="label" data-i18n="view.divc.card.rating">Sustainability</div>
                    <div class="value ${ratingCls}">${ratingLabel}</div></div>
                <div class="card"><div class="label" data-i18n="view.divc.card.coverage">Earnings coverage</div>
                    <div class="value">${coverage}</div></div>
                <div class="card"><div class="label" data-i18n="view.divc.card.retention">Retention</div>
                    <div class="value">${pct(r.retention_ratio_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.divc.card.fcf">FCF payout</div>
                    <div class="value">${fcf}</div></div>
            </div>
        </div>
    `;
    applyUiI18n(el);
}
