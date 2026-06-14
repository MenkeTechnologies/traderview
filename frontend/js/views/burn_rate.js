// Burn rate & runway — gross/net burn, months of cash, and months to
// break-even given revenue growth, via /calc/burn-rate. Updates live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['cash_on_hand_usd', 'Cash on hand ($)', 100000],
    ['monthly_revenue_usd', 'Monthly revenue ($)', 5000],
    ['monthly_expenses_usd', 'Monthly expenses ($)', 10000],
    ['monthly_revenue_growth_pct', 'Revenue growth (%/mo)', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VIEW = 'burn-rate';
let lastReport = null;
let lastBody = null;

export async function renderBurnRate(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.burn.h1.title">// BURN RATE & RUNWAY</span></h1>
        <p class="muted small" data-i18n="view.burn.hint.intro">
            How long the cash lasts. Gross burn is monthly expenses; net burn is expenses minus
            revenue — what actually drains the bank. Runway is the months of cash at that burn,
            simulated month by month so growing revenue extends it. If revenue overtakes
            expenses before the cash runs out, you reach break-even and never deplete. Updates
            as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.burn.h2.inputs">The numbers</h2>
            <form id="burn-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.burn.label.${key}">${label}</span>
                        <input type="number" step="0.01" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="burn-tools" class="ce-toolbar"></div>
            <button type="button" id="burn-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="burn-sens" class="ce-sens"></div>
        </div>
        <div id="burn-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#burn-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        return body;
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcBurnRate(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.burn.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#burn-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'burn-rate.csv' });
    mount.querySelector('#burn-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['gross_burn_usd', r.gross_burn_usd],
        ['net_burn_usd', r.net_burn_usd],
        ['runway_months', r.runway_months == null ? '' : r.runway_months],
        ['months_to_breakeven', r.months_to_breakeven == null ? '' : r.months_to_breakeven],
        ['already_profitable', r.already_profitable],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#burn-result');
    let runwayVal, runwayCls;
    if (r.already_profitable) {
        runwayVal = t('view.burn.profitable');
        runwayCls = 'pos';
    } else if (r.runway_months == null) {
        runwayVal = t('view.burn.reaches_breakeven');
        runwayCls = 'pos';
    } else {
        runwayVal = `${r.runway_months} ${t('view.burn.months')}`;
        runwayCls = r.runway_months <= 6 ? 'neg' : r.runway_months <= 12 ? '' : 'pos';
    }
    const breakeven = r.already_profitable
        ? t('view.burn.now')
        : r.months_to_breakeven == null
            ? '—'
            : `${r.months_to_breakeven} ${t('view.burn.months')}`;
    // Line chart: net burn as monthly revenue sweeps 0 → 2× expenses (crosses zero at break-even).
    const cap = (body.monthly_expenses_usd || 10000) * 2;
    const xs = enh.linspace(0, cap, 13);
    const pts = await Promise.all(xs.map(async (rev) => {
        const rr = await api.calcBurnRate({ ...body, monthly_revenue_usd: rev, monthly_revenue_growth_pct: 0 });
        return { x: rev / 1000, y: rr ? rr.net_burn_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'revenue $k', ylabel: 'net burn $' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.burn.h2.result">The runway</h2>
            <div class="cards">
                <div class="card ${runwayCls}"><div class="label" data-i18n="view.burn.card.runway">Runway</div>
                    <div class="value ${runwayCls}">${runwayVal}</div></div>
                <div class="card ${r.net_burn_usd > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.burn.card.net">Net burn / mo</div>
                    <div class="value">${money(r.net_burn_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.burn.card.gross">Gross burn / mo</div>
                    <div class="value">${money(r.gross_burn_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.burn.card.breakeven">Months to break-even</div>
                    <div class="value">${breakeven}</div></div>
            </div>
            ${chart}
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#burn-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: monthly expenses 0.5× → 1.5× current; y: monthly revenue 0 → 1.5× current. Output: net burn.
    const ex = base.monthly_expenses_usd || 10000;
    const rv = base.monthly_revenue_usd || 5000;
    const xVals = enh.linspace(ex * 0.5, ex * 1.5, 5);
    const yVals = enh.linspace(0, Math.max(rv, ex) * 1.5, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'monthly_expenses_usd', yKey: 'monthly_revenue_usd', xVals, yVals, compute: (b) => api.calcBurnRate(b), pick: (r) => (r ? r.net_burn_usd : null) });
    if (!viewIsCurrent(tok)) return;
    // Lower net burn is better, so invert the value for shading via a negated formatter base:
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : '$' + Math.round(-v)), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.burn.label.monthly_expenses_usd') || 'Expenses', yName: t('view.burn.label.monthly_revenue_usd') || 'Revenue' });
}
