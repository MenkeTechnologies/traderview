// IRC § 4501 — Stock Buyback Excise Tax (IRA 2022).
// 1% excise tax on FMV of publicly traded corp stock repurchased.
// Effective for repurchases after 12/31/2022.
// Net of issuances: NEW stock issued in compensation, M&A, conversions REDUCES base.
// Exceptions: dividend treatment, ESOP contributions, ≤ $1M de minimis, RIC/REIT.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    fmv_repurchased: 0,
    stock_issued_compensation: 0,
    stock_issued_acquisition: 0,
    treasury_to_employee: 0,
    fair_market_dividend_treatment: 0,
    esop_contributions: 0,
    is_public: true,
    is_ric_reit: false,
    post_2022: true,
    proposed_rate_increase: false,
};

export async function renderSection4501(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4501.h1.title">// § 4501 STOCK BUYBACK EXCISE</span></h1>
        <p class="muted small" data-i18n="view.s4501.hint.intro">
            <strong>1% excise tax on FMV</strong> of publicly traded corp stock REPURCHASED, effective after
            12/31/2022 (IRA 2022). <strong>Net of issuances:</strong> NEW stock issued reduces base (compensation,
            M&A, conversions). <strong>Exceptions:</strong> dividend treatment, ESOP contributions, ≤ $1M de
            minimis, RIC/REIT, reorganizations. <strong>Pending proposal:</strong> rate increase to 4% under
            various legislative drafts. Filed on <strong>Form 7208</strong>.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4501.h2.inputs">Inputs</h2>
            <form id="s4501-form" class="inline-form">
                <label><span data-i18n="view.s4501.label.fmv">FMV repurchased ($)</span>
                    <input type="number" step="0.01" name="fmv_repurchased" value="${state.fmv_repurchased}"></label>
                <label><span data-i18n="view.s4501.label.comp">Stock issued — compensation ($)</span>
                    <input type="number" step="0.01" name="stock_issued_compensation" value="${state.stock_issued_compensation}"></label>
                <label><span data-i18n="view.s4501.label.acquisition">Stock issued — acquisition ($)</span>
                    <input type="number" step="0.01" name="stock_issued_acquisition" value="${state.stock_issued_acquisition}"></label>
                <label><span data-i18n="view.s4501.label.treasury">Treasury → employee benefit ($)</span>
                    <input type="number" step="0.01" name="treasury_to_employee" value="${state.treasury_to_employee}"></label>
                <label><span data-i18n="view.s4501.label.dividend">Treated as dividend by recipient ($)</span>
                    <input type="number" step="0.01" name="fair_market_dividend_treatment" value="${state.fair_market_dividend_treatment}"></label>
                <label><span data-i18n="view.s4501.label.esop">ESOP contributions ($)</span>
                    <input type="number" step="0.01" name="esop_contributions" value="${state.esop_contributions}"></label>
                <label><span data-i18n="view.s4501.label.public">Publicly traded?</span>
                    <input type="checkbox" name="is_public" ${state.is_public ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4501.label.ric">RIC / REIT?</span>
                    <input type="checkbox" name="is_ric_reit" ${state.is_ric_reit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4501.label.post_2022">Post-2022?</span>
                    <input type="checkbox" name="post_2022" ${state.post_2022 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4501.label.rate_increase">Assume proposed 4% rate?</span>
                    <input type="checkbox" name="proposed_rate_increase" ${state.proposed_rate_increase ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s4501.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4501-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4501.h2.coverage">Coverage scope</h2>
            <ul class="muted small">
                <li data-i18n="view.s4501.cov.public">Covered: publicly traded US corp (also "covered surrogate foreign corp" under § 7874)</li>
                <li data-i18n="view.s4501.cov.private">Excluded: privately held companies (no securities exchange listing)</li>
                <li data-i18n="view.s4501.cov.expat">Foreign acquirer's US subsidiary repurchases of foreign parent stock — included if reach $1M</li>
                <li data-i18n="view.s4501.cov.ric_reit">RICs (mutual funds) + REITs exempt</li>
                <li data-i18n="view.s4501.cov.tender">Self-tender offers + market repurchases + accelerated share repurchases — all covered</li>
                <li data-i18n="view.s4501.cov.adr">ADR programs: complex sourcing rules</li>
                <li data-i18n="view.s4501.cov.derivative">Substantially similar transactions via derivatives — covered (anti-abuse)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4501.h2.exceptions">§ 4501 exceptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s4501.exc.dividend">Dividend treatment: reduces base by amount treated as dividend by recipient</li>
                <li data-i18n="view.s4501.exc.reorg">Reorgs § 368 (qualified): exception when no FMV exchange</li>
                <li data-i18n="view.s4501.exc.esop">Contributions to employer-sponsored ESOP: full offset</li>
                <li data-i18n="view.s4501.exc.bank_dealer">Bank / Securities Dealer in ordinary course</li>
                <li data-i18n="view.s4501.exc.de_minimis">$1M de minimis: full exemption when annual buybacks ≤ $1M</li>
                <li data-i18n="view.s4501.exc.netting">"Netting rule": new issuances (compensation + acquisition) reduce base $-for-$</li>
                <li data-i18n="view.s4501.exc.spinoff">§ 355 spinoffs: distributing corp stock not subject</li>
                <li data-i18n="view.s4501.exc.gh_compete">Non-deductible: § 162 deduction unavailable for excise tax</li>
            </ul>
        </div>
    `;
    document.getElementById('s4501-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.fmv_repurchased = Number(fd.get('fmv_repurchased')) || 0;
        state.stock_issued_compensation = Number(fd.get('stock_issued_compensation')) || 0;
        state.stock_issued_acquisition = Number(fd.get('stock_issued_acquisition')) || 0;
        state.treasury_to_employee = Number(fd.get('treasury_to_employee')) || 0;
        state.fair_market_dividend_treatment = Number(fd.get('fair_market_dividend_treatment')) || 0;
        state.esop_contributions = Number(fd.get('esop_contributions')) || 0;
        state.is_public = !!fd.get('is_public');
        state.is_ric_reit = !!fd.get('is_ric_reit');
        state.post_2022 = !!fd.get('post_2022');
        state.proposed_rate_increase = !!fd.get('proposed_rate_increase');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4501-output');
    if (!el) return;
    const isCovered = state.is_public && !state.is_ric_reit && state.post_2022;
    const deMinimis = state.fmv_repurchased <= 1_000_000;
    const netIssuances = state.stock_issued_compensation + state.stock_issued_acquisition + state.treasury_to_employee + state.fair_market_dividend_treatment + state.esop_contributions;
    const taxBase = Math.max(0, state.fmv_repurchased - netIssuances);
    const rate = state.proposed_rate_increase ? 0.04 : 0.01;
    const exciseTax = (isCovered && !deMinimis) ? taxBase * rate : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4501.h2.result">§ 4501 excise computation</h2>
            <div class="cards">
                <div class="card ${isCovered ? '' : 'pos'}">
                    <div class="label" data-i18n="view.s4501.card.covered">Covered taxpayer?</div>
                    <div class="value">${isCovered ? esc(t('view.s4501.status.yes')) : esc(t('view.s4501.status.no'))}</div>
                </div>
                <div class="card ${deMinimis ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s4501.card.de_minimis">De minimis ≤ $1M</div>
                    <div class="value">${deMinimis ? esc(t('view.s4501.status.yes')) : esc(t('view.s4501.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4501.card.fmv">FMV repurchased</div>
                    <div class="value">$${state.fmv_repurchased.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s4501.card.netting">Net issuances offset</div>
                    <div class="value">$${netIssuances.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4501.card.base">Tax base (net)</div>
                    <div class="value">$${taxBase.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4501.card.rate">Rate</div>
                    <div class="value">${(rate * 100).toFixed(0)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4501.card.excise">Excise tax</div>
                    <div class="value">$${exciseTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.proposed_rate_increase ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s4501.proposal_note">
                    Proposed 4% rate scenario. Various Biden-Harris + congressional proposals quadruple
                    the rate from 1% to 4%. Plan for higher rate by netting more issuances + ESOP contributions.
                </p>
            ` : ''}
        </div>
    `;
}
