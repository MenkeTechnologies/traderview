// Piotroski F-Score — 9-point financial-strength test across two years, via
// /calc/piotroski-f-score. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

// Each input: [name, i18n key, default this-year, default prior-year].
// A single value (no prior) uses one column.
const PAIRS = [
    ['net_income', 'view.piotroski.label.ni', 100, 50],
    ['total_assets', 'view.piotroski.label.ta', 1000, 1000],
    ['long_term_debt', 'view.piotroski.label.ltd', 100, 200],
    ['current_assets', 'view.piotroski.label.ca', 400, 300],
    ['current_liabilities', 'view.piotroski.label.cl', 200, 200],
    ['shares_outstanding', 'view.piotroski.label.shares', 1000, 1000],
    ['gross_profit', 'view.piotroski.label.gp', 600, 400],
    ['revenue', 'view.piotroski.label.rev', 1000, 800],
];

const CRITERIA = [
    ['roa_positive', 'view.piotroski.crit.roapos'],
    ['cfo_positive', 'view.piotroski.crit.cfopos'],
    ['roa_rising', 'view.piotroski.crit.roarising'],
    ['accruals_quality', 'view.piotroski.crit.accruals'],
    ['leverage_falling', 'view.piotroski.crit.leverage'],
    ['liquidity_rising', 'view.piotroski.crit.liquidity'],
    ['no_dilution', 'view.piotroski.crit.dilution'],
    ['margin_rising', 'view.piotroski.crit.margin'],
    ['turnover_rising', 'view.piotroski.crit.turnover'],
];

const RATING_CLS = { strong: 'pos', moderate: '', weak: 'neg' };

export async function renderPiotroskiFScore(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    const pairRows = PAIRS.map(([n, k, dt, dty]) => `
        <div class="piotroski-row">
            <span class="piotroski-label" data-i18n="${k}">${n}</span>
            <input type="number" step="0.01" name="${n}_t" value="${dt}" required>
            <input type="number" step="0.01" name="${n}_ty" value="${dty}" required>
        </div>`).join('');
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.piotroski.h1.title">// PIOTROSKI F-SCORE</span></h1>
        <p class="muted small" data-i18n="view.piotroski.hint.intro">
            Joseph Piotroski's 9-point test of financial strength — one point for each criterion
            passed across profitability, leverage/liquidity, and operating efficiency. 8–9 is
            strong, 0–2 is weak. Enter this year and last year for each line; operating cash flow
            is this year only. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.piotroski.h2.inputs">The financials</h2>
            <form id="piotroski-form">
                <div class="piotroski-grid">
                    <div class="piotroski-head">
                        <span></span>
                        <span data-i18n="view.piotroski.col.thisyear">This year</span>
                        <span data-i18n="view.piotroski.col.prioryear">Prior year</span>
                    </div>
                    ${pairRows}
                    <div class="piotroski-row">
                        <span class="piotroski-label" data-i18n="view.piotroski.label.cfo">Operating cash flow</span>
                        <input type="number" step="0.01" name="operating_cash_flow_t" value="150" required>
                        <span class="muted small">—</span>
                    </div>
                </div>
            </form>
        </div>
        <div id="piotroski-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#piotroski-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = { operating_cash_flow_t: Number(fd.get('operating_cash_flow_t')) || 0 };
        for (const [n] of PAIRS) {
            body[`${n}_t`] = Number(fd.get(`${n}_t`)) || 0;
            body[`${n}_ty`] = Number(fd.get(`${n}_ty`)) || 0;
        }
        try {
            const r = await api.calcPiotroskiFScore(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.piotroski.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#piotroski-result');
    const cls = RATING_CLS[r.rating] ?? '';
    const rows = CRITERIA.map(([key, i18n]) => {
        const pass = r[key];
        return `<tr class="${pass ? 'pos' : ''}"><td data-i18n="${i18n}">${key}</td><td>${pass ? t('view.piotroski.pass') : t('view.piotroski.fail')}</td></tr>`;
    }).join('');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.piotroski.h2.result">The score</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="view.piotroski.card.score">F-Score</div>
                    <div class="value ${cls}">${r.f_score} / 9</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.piotroski.card.rating">Rating</div>
                    <div class="value ${cls}" data-i18n="view.piotroski.rating.${r.rating}">—</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.piotroski.col.criterion">Criterion</th><th data-i18n="view.piotroski.col.result">Result</th></tr></thead>
                <tbody>${rows}</tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
