// Home Office Deduction Calculator — IRC § 280A.
// Two methods: simplified ($5/sqft up to 300sqft = $1,500 cap) vs. actual
// expenses (utilities, rent/mortgage interest, depreciation × business %).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const LS_KEY = 'tv-home-office-v1';
const SIMPLIFIED_RATE_PER_SQFT = 5;
const SIMPLIFIED_MAX_SQFT = 300;
const SIMPLIFIED_CAP = 1_500;

function loadState() {
    try {
        const raw = localStorage.getItem(LS_KEY);
        return raw ? JSON.parse(raw) : null;
    } catch { return null; }
}
function saveState(s) {
    try { localStorage.setItem(LS_KEY, JSON.stringify(s)); } catch { /* private mode */ }
}

let state = loadState() || {
    method: 'compare',
    office_sqft: 100,
    home_sqft: 1500,
    rent_or_mortgage_interest: 0,
    utilities: 0,
    insurance: 0,
    repairs_general: 0,
    repairs_office: 0,
    depreciation: 0,
    biz_income: 0,
};

export async function renderHomeOffice(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.home_office.h1.title">// HOME OFFICE DEDUCTION</span></h1>
        <p class="muted small" data-i18n="view.home_office.hint.intro">
            IRC § 280A — deduct a portion of home expenses for the area regularly +
            exclusively used for business. Method compare side-by-side; pick whichever
            yields a larger deduction (you can change methods year-to-year).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.home_office.h2.inputs">Inputs</h2>
            <form id="ho-form" class="inline-form">
                <label><span data-i18n="view.home_office.label.office_sqft">Office sqft</span>
                    <input type="number" step="1" name="office_sqft" value="${state.office_sqft}" min="1"></label>
                <label><span data-i18n="view.home_office.label.home_sqft">Total home sqft</span>
                    <input type="number" step="1" name="home_sqft" value="${state.home_sqft}" min="1"></label>
                <label><span data-i18n="view.home_office.label.rent_mortgage">Rent OR mortgage interest ($/yr)</span>
                    <input type="number" step="0.01" name="rent_or_mortgage_interest" value="${state.rent_or_mortgage_interest}"></label>
                <label><span data-i18n="view.home_office.label.utilities">Utilities ($/yr)</span>
                    <input type="number" step="0.01" name="utilities" value="${state.utilities}"></label>
                <label><span data-i18n="view.home_office.label.insurance">Home insurance ($/yr)</span>
                    <input type="number" step="0.01" name="insurance" value="${state.insurance}"></label>
                <label><span data-i18n="view.home_office.label.repairs_general">General repairs ($/yr)</span>
                    <input type="number" step="0.01" name="repairs_general" value="${state.repairs_general}"></label>
                <label><span data-i18n="view.home_office.label.repairs_office">Office-only repairs ($/yr)</span>
                    <input type="number" step="0.01" name="repairs_office" value="${state.repairs_office}"></label>
                <label><span data-i18n="view.home_office.label.depreciation">Depreciation ($/yr) — homeowner only</span>
                    <input type="number" step="0.01" name="depreciation" value="${state.depreciation}"></label>
                <label><span data-i18n="view.home_office.label.biz_income">Net business income (for cap)</span>
                    <input type="number" step="0.01" name="biz_income" value="${state.biz_income}"></label>
                <button class="primary" type="submit" data-i18n="view.home_office.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="ho-output"></div>
    `;
    document.getElementById('ho-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state = {
            ...state,
            office_sqft: Number(fd.get('office_sqft')) || 0,
            home_sqft: Number(fd.get('home_sqft')) || 1,
            rent_or_mortgage_interest: Number(fd.get('rent_or_mortgage_interest')) || 0,
            utilities: Number(fd.get('utilities')) || 0,
            insurance: Number(fd.get('insurance')) || 0,
            repairs_general: Number(fd.get('repairs_general')) || 0,
            repairs_office: Number(fd.get('repairs_office')) || 0,
            depreciation: Number(fd.get('depreciation')) || 0,
            biz_income: Number(fd.get('biz_income')) || 0,
        };
        saveState(state);
        render();
    });
    render();
}

function render() {
    const el = document.getElementById('ho-output');
    if (!el) return;
    const officePct = state.office_sqft / Math.max(1, state.home_sqft);
    // Simplified method: $5/sqft × min(office_sqft, 300), capped at $1500.
    const simplified = Math.min(state.office_sqft, SIMPLIFIED_MAX_SQFT) * SIMPLIFIED_RATE_PER_SQFT;
    const simplifiedCapped = Math.min(simplified, SIMPLIFIED_CAP);
    // Actual method: prorated indirect expenses + direct office repairs.
    const indirect = state.rent_or_mortgage_interest
        + state.utilities + state.insurance + state.repairs_general + state.depreciation;
    const actualBeforeCap = indirect * officePct + state.repairs_office;
    // Both methods are capped at biz income.
    const cap = Math.max(0, state.biz_income);
    const simplifiedAfterCap = Math.min(simplifiedCapped, cap);
    const actualAfterCap = Math.min(actualBeforeCap, cap);
    const better = actualAfterCap > simplifiedAfterCap ? 'actual' : 'simplified';
    el.innerHTML = `
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.home_office.h2.simplified">Simplified method</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.home_office.row.eligible_sqft">Eligible sqft (max 300)</td>
                        <td>${Math.min(state.office_sqft, SIMPLIFIED_MAX_SQFT)}</td></tr>
                    <tr><td data-i18n="view.home_office.row.rate">Rate</td>
                        <td>$5/sqft</td></tr>
                    <tr><td data-i18n="view.home_office.row.deduction_pre_cap">Pre-cap deduction</td>
                        <td>$${simplifiedCapped.toFixed(2)}</td></tr>
                    <tr><td><strong data-i18n="view.home_office.row.deduction_post_cap">After biz-income cap</strong></td>
                        <td class="pos"><strong>$${simplifiedAfterCap.toFixed(2)}</strong></td></tr>
                </tbody></table>
                <p class="muted small" data-i18n="view.home_office.simplified.note">
                    No depreciation recapture. Easiest method — no records of utilities etc. needed.
                </p>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.home_office.h2.actual">Actual expense method</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.home_office.row.office_pct">Office %</td>
                        <td>${(officePct * 100).toFixed(2)}%</td></tr>
                    <tr><td data-i18n="view.home_office.row.indirect_total">Indirect expenses (rent/util/ins/dep)</td>
                        <td>$${indirect.toFixed(2)}</td></tr>
                    <tr><td data-i18n="view.home_office.row.prorated_indirect">Prorated indirect</td>
                        <td>$${(indirect * officePct).toFixed(2)}</td></tr>
                    <tr><td data-i18n="view.home_office.row.direct_repairs">Direct office repairs</td>
                        <td>$${state.repairs_office.toFixed(2)}</td></tr>
                    <tr><td data-i18n="view.home_office.row.deduction_pre_cap_2">Pre-cap deduction</td>
                        <td>$${actualBeforeCap.toFixed(2)}</td></tr>
                    <tr><td><strong data-i18n="view.home_office.row.deduction_post_cap_2">After biz-income cap</strong></td>
                        <td class="pos"><strong>$${actualAfterCap.toFixed(2)}</strong></td></tr>
                </tbody></table>
                <p class="muted small" data-i18n="view.home_office.actual.note">
                    Higher upside but depreciation reduces home cost basis (gain recaptured on sale).
                    Requires receipts + utility records.
                </p>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.home_office.h2.recommendation">Recommendation</h2>
            <p>
                <strong>${esc(t('view.home_office.recommend.' + better))}</strong> ·
                ${esc(t('view.home_office.recommend.savings', {
                    diff: Math.abs(actualAfterCap - simplifiedAfterCap).toFixed(2),
                    better: t('view.home_office.method.' + better),
                }))}
            </p>
            <p class="muted small" data-i18n="view.home_office.disclaimer">
                Office must be regularly + exclusively used for business. Daycare + storage exceptions exist.
                Excess deduction (above biz income) carries over only under actual method.
            </p>
        </div>
    `;
}
