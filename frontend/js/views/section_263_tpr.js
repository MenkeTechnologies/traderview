// § 263 + Tangible Property Regulations — Repair vs Improvement classifier.
// Repairs deduct currently (§ 162). Improvements capitalize (§ 263). TPRs (Treas. Reg.
// § 1.263(a)-3) provide tests: B (Betterment), A (Adaptation), R (Restoration) — BAR.
// Three safe harbors: de minimis ($2.5k / $5k AFS), routine maintenance (10-yr cycle),
// small taxpayer (≤ $10M gross + 2% basis OR $10k cap).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const DE_MINIMIS_NO_AFS = 2_500;
const DE_MINIMIS_AFS = 5_000;
const SMALL_TAXPAYER_GROSS_RECEIPTS = 10_000_000;
const SMALL_TAXPAYER_BASIS_PCT = 0.02;
const SMALL_TAXPAYER_CAP = 10_000;
const ROUTINE_MAINT_CYCLE_YEARS = 10;

let state = {
    expenditure_amount: 0,
    has_afs: false,
    property_unadjusted_basis: 0,
    gross_receipts: 0,
    expected_recurrence_years: 0,
    is_betterment: false,
    is_adaptation: false,
    is_restoration: false,
    is_personal_property: true,
    marginal_rate: 0.32,
};

export async function renderSection263Tpr(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tpr.h1.title">// § 263 + TPRs REPAIR VS IMPROVEMENT</span></h1>
        <p class="muted small" data-i18n="view.tpr.hint.intro">
            Repairs deduct currently (§ 162). Improvements capitalize (§ 263). TPRs apply
            <strong>BAR test: Betterment, Adaptation, Restoration</strong>. Plus 3 safe harbors:
            <strong>De Minimis ($2.5k / $5k AFS)</strong>, <strong>Routine Maintenance
            (10-yr cycle)</strong>, <strong>Small Taxpayer Safe Harbor (≤ $10M gross + 2% basis
            OR $10k)</strong>. Annual elections required.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.tpr.h2.inputs">Inputs</h2>
            <form id="tpr-form" class="inline-form">
                <label><span data-i18n="view.tpr.label.amount">Expenditure amount ($)</span>
                    <input type="number" step="10" name="expenditure_amount" value="${state.expenditure_amount}"></label>
                <label><span data-i18n="view.tpr.label.afs">Applicable Financial Statement (AFS)?</span>
                    <input type="checkbox" name="has_afs" ${state.has_afs ? 'checked' : ''}></label>
                <label><span data-i18n="view.tpr.label.basis">Unadjusted property basis ($)</span>
                    <input type="number" step="1000" name="property_unadjusted_basis" value="${state.property_unadjusted_basis}"></label>
                <label><span data-i18n="view.tpr.label.gross">Gross receipts (3-yr avg) ($)</span>
                    <input type="number" step="100000" name="gross_receipts" value="${state.gross_receipts}"></label>
                <label><span data-i18n="view.tpr.label.recurrence">Expected recurrence (years)</span>
                    <input type="number" step="1" name="expected_recurrence_years" value="${state.expected_recurrence_years}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.tpr.label.betterment">Betterment (material defect / addition / capacity)?</span>
                    <input type="checkbox" name="is_betterment" ${state.is_betterment ? 'checked' : ''}></label>
                <label><span data-i18n="view.tpr.label.adaptation">Adaptation (new / different use)?</span>
                    <input type="checkbox" name="is_adaptation" ${state.is_adaptation ? 'checked' : ''}></label>
                <label><span data-i18n="view.tpr.label.restoration">Restoration (major component / rebuild / from disrepair)?</span>
                    <input type="checkbox" name="is_restoration" ${state.is_restoration ? 'checked' : ''}></label>
                <label><span data-i18n="view.tpr.label.personal">Personal property (vs real)?</span>
                    <input type="checkbox" name="is_personal_property" ${state.is_personal_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.tpr.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.tpr.btn.classify">Classify</button>
            </form>
        </div>
        <div id="tpr-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.tpr.h2.bar">BAR Test (Improvement)</h2>
            <ul class="muted small">
                <li data-i18n="view.tpr.bar.betterment"><strong>B</strong>etterment: ameliorates material defect from acquisition / addition / increase in capacity / productivity / efficiency / quality / strength</li>
                <li data-i18n="view.tpr.bar.adaptation"><strong>A</strong>daptation: changes property to new / different use not consistent with intended use at acquisition</li>
                <li data-i18n="view.tpr.bar.restoration"><strong>R</strong>estoration: replaces major component / rebuilds to like-new / from disrepair / claimed casualty loss / replaces UoP that has substantially exhausted recovery period</li>
                <li data-i18n="view.tpr.bar.unit">Apply BAR per "unit of property" — for buildings = 9 building systems (HVAC, plumbing, electrical, elevator, fire, security, gas, plumbing, structural)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.tpr.h2.elections">Required annual elections</h2>
            <ul class="muted small">
                <li data-i18n="view.tpr.elect.de_minimis">De minimis safe harbor: annual policy statement + written accounting policy + invoice-level expensing</li>
                <li data-i18n="view.tpr.elect.routine">Routine maintenance: business expectation that activity will recur within 10 yrs (buildings) or class life (personal)</li>
                <li data-i18n="view.tpr.elect.small_tp">Small taxpayer safe harbor: § 263(a) attach statement; building unadjusted basis ≤ $1M required</li>
                <li data-i18n="view.tpr.elect.partial_disp">Partial Disposition election (§ 1.168(i)-8): replace component, dispose of old basis</li>
                <li data-i18n="view.tpr.elect.materials">Materials & supplies (Reg § 1.162-3): de minimis $200 or 12-month rule</li>
            </ul>
        </div>
    `;
    document.getElementById('tpr-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.expenditure_amount = Number(fd.get('expenditure_amount')) || 0;
        state.has_afs = !!fd.get('has_afs');
        state.property_unadjusted_basis = Number(fd.get('property_unadjusted_basis')) || 0;
        state.gross_receipts = Number(fd.get('gross_receipts')) || 0;
        state.expected_recurrence_years = Number(fd.get('expected_recurrence_years')) || 0;
        state.is_betterment = !!fd.get('is_betterment');
        state.is_adaptation = !!fd.get('is_adaptation');
        state.is_restoration = !!fd.get('is_restoration');
        state.is_personal_property = !!fd.get('is_personal_property');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('tpr-output');
    if (!el) return;
    const deMinimisLimit = state.has_afs ? DE_MINIMIS_AFS : DE_MINIMIS_NO_AFS;
    const passesDeMinimis = state.expenditure_amount <= deMinimisLimit;
    const isRoutineMaint = state.expected_recurrence_years > 0 && state.expected_recurrence_years <= ROUTINE_MAINT_CYCLE_YEARS && !state.is_betterment && !state.is_adaptation;
    const smallTpQualifies = state.gross_receipts <= SMALL_TAXPAYER_GROSS_RECEIPTS;
    const smallTpCap = Math.min(state.property_unadjusted_basis * SMALL_TAXPAYER_BASIS_PCT, SMALL_TAXPAYER_CAP);
    const passesSmallTp = smallTpQualifies && state.expenditure_amount <= smallTpCap;
    const isImprovement = state.is_betterment || state.is_adaptation || state.is_restoration;
    let classification, classCls;
    if (passesDeMinimis) { classification = 'view.tpr.class.de_minimis'; classCls = 'pos'; }
    else if (isRoutineMaint) { classification = 'view.tpr.class.routine'; classCls = 'pos'; }
    else if (passesSmallTp) { classification = 'view.tpr.class.small_tp'; classCls = 'pos'; }
    else if (isImprovement) { classification = 'view.tpr.class.improvement'; classCls = 'neg'; }
    else { classification = 'view.tpr.class.repair'; classCls = 'pos'; }
    const isDeductibleNow = classCls === 'pos';
    const year1Saving = isDeductibleNow ? state.expenditure_amount * state.marginal_rate : 0;
    const cashflowDifference = state.expenditure_amount * state.marginal_rate * (isDeductibleNow ? 1 : 0.10);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.tpr.h2.result">Classification</h2>
            <div class="cards">
                <div class="card ${classCls}">
                    <div class="label" data-i18n="view.tpr.card.classification">Classification</div>
                    <div class="value">${esc(t(classification))}</div>
                </div>
                <div class="card ${passesDeMinimis ? 'pos' : ''}">
                    <div class="label" data-i18n="view.tpr.card.de_minimis_limit">De minimis limit</div>
                    <div class="value">$${deMinimisLimit.toLocaleString()}</div>
                </div>
                <div class="card ${isRoutineMaint ? 'pos' : ''}">
                    <div class="label" data-i18n="view.tpr.card.routine">Routine maintenance?</div>
                    <div class="value">${isRoutineMaint ? esc(t('view.tpr.status.yes')) : esc(t('view.tpr.status.no'))}</div>
                </div>
                <div class="card ${passesSmallTp ? 'pos' : ''}">
                    <div class="label" data-i18n="view.tpr.card.small_tp">Small TP safe harbor</div>
                    <div class="value">${passesSmallTp ? esc(t('view.tpr.status.yes')) : esc(t('view.tpr.status.no'))}</div>
                </div>
                <div class="card ${isImprovement ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.tpr.card.is_improvement">BAR triggers improvement?</div>
                    <div class="value">${isImprovement ? esc(t('view.tpr.status.yes')) : esc(t('view.tpr.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.tpr.card.y1_saving">Year-1 tax savings if deductible</div>
                    <div class="value">$${year1Saving.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
