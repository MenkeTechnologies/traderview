// IRC § 1361 — S-corp election + Reasonable Compensation tracker.
// Form 2553 deadline: 2 months + 15 days into election year (or anytime in prior year).
// Late election: Rev. Proc. 2013-30 auto-relief if reasonable cause + <3 yr 75 day window.
// Reasonable comp = arms-length W-2 wages BEFORE distributions can be taken.
// IRS challenges: David E. Watson, P.C. (8th Cir. 2012) — $24k comp on $200k profits = lost.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    business_type: 'llc_s_election',
    formation_date: '',
    desired_effective_date: '',
    gross_revenue: 0,
    net_profit_before_owner_comp: 0,
    owner_w2_wages_planned: 0,
    industry: 'professional_services',
    market_rate_compensation: 0,
    self_employment_tax_rate: 0.153,
    federal_marginal_rate: 0.32,
};

export async function renderSection1361(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1361.h1.title">// § 1361 S-CORP + REASONABLE COMP</span></h1>
        <p class="muted small" data-i18n="view.s1361.hint.intro">
            Form 2553 election: <strong>2 months 15 days into election year</strong> (Mar 15
            for calendar). LATE election? Rev. Proc. 2013-30 auto-relief if reasonable cause + within
            3-yr 75-day window. <strong>Reasonable Compensation:</strong> W-2 wages must be
            arms-length BEFORE distributions. Saves SE tax (15.3%) on profit ABOVE reasonable comp.
            <strong>Audit risk:</strong> Watson 8th Cir. lost $24k comp on $200k profit case.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1361.h2.election_timing">Election timing</h2>
            <ul class="muted small">
                <li data-i18n="view.s1361.elect.deadline">Form 2553 due by Mar 15 of effective year (calendar) OR within 2 mo 15 days of inception</li>
                <li data-i18n="view.s1361.elect.late">Late election: Rev. Proc. 2013-30 auto-relief if &lt; 3 yr 75 days + reasonable cause</li>
                <li data-i18n="view.s1361.elect.consent">All shareholders must sign / spouse must consent if community property</li>
                <li data-i18n="view.s1361.elect.eligible">Eligible: 100 shareholder cap, 1 class of stock, no foreign owners, no C-corp/LLC owners</li>
                <li data-i18n="view.s1361.elect.trusts">Trust shareholders: ESBT or QSST election required</li>
                <li data-i18n="view.s1361.elect.tax_year">Calendar year required unless § 444 election + back-up payment</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1361.h2.inputs">Inputs</h2>
            <form id="s1361-form" class="inline-form">
                <label><span data-i18n="view.s1361.label.gross_revenue">Gross revenue ($)</span>
                    <input type="number" step="0.01" name="gross_revenue" value="${state.gross_revenue}"></label>
                <label><span data-i18n="view.s1361.label.net_profit">Net profit before owner comp ($)</span>
                    <input type="number" step="0.01" name="net_profit_before_owner_comp" value="${state.net_profit_before_owner_comp}"></label>
                <label><span data-i18n="view.s1361.label.w2_planned">Owner W-2 wages planned ($)</span>
                    <input type="number" step="0.01" name="owner_w2_wages_planned" value="${state.owner_w2_wages_planned}"></label>
                <label><span data-i18n="view.s1361.label.industry">Industry</span>
                    <select name="industry">
                        <option value="trader" ${state.industry === 'trader' ? 'selected' : ''}>Trader (TTS)</option>
                        <option value="professional_services" ${state.industry === 'professional_services' ? 'selected' : ''}>Professional services</option>
                        <option value="consulting" ${state.industry === 'consulting' ? 'selected' : ''}>Consulting</option>
                        <option value="construction" ${state.industry === 'construction' ? 'selected' : ''}>Construction</option>
                        <option value="real_estate" ${state.industry === 'real_estate' ? 'selected' : ''}>Real estate</option>
                        <option value="ecommerce" ${state.industry === 'ecommerce' ? 'selected' : ''}>E-commerce</option>
                        <option value="medical" ${state.industry === 'medical' ? 'selected' : ''}>Medical / dental</option>
                        <option value="other" ${state.industry === 'other' ? 'selected' : ''}>Other</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1361.label.market_rate">Market rate comp (BLS / RC Reports) ($)</span>
                    <input type="number" step="0.01" name="market_rate_compensation" value="${state.market_rate_compensation}"></label>
                <label><span data-i18n="view.s1361.label.fed_marginal">Federal marginal %</span>
                    <input type="number" step="0.01" name="federal_marginal_rate" value="${state.federal_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s1361.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1361-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1361.h2.case_law">Reasonable comp case law</h2>
            <ul class="muted small">
                <li data-i18n="view.s1361.case.watson">Watson, P.C. v. United States (8th Cir. 2012) — DENIED $24k on $200k+ profit; reclassified $91k as wages</li>
                <li data-i18n="view.s1361.case.sean">Sean McAlary Ltd. v. Comm'r (2013) — DENIED $24k for real estate agent; reclassified $83k</li>
                <li data-i18n="view.s1361.case.glass_blocks">Glass Blocks Unlimited v. Comm'r (2013) — DENIED $30k on $700k revenue</li>
                <li data-i18n="view.s1361.case.davis">Davis v. United States (1976, 11th Cir.) — established multi-factor framework</li>
                <li data-i18n="view.s1361.case.fact_sheet">IRS Fact Sheet 2008-25: 9 factors for reasonable comp determination</li>
            </ul>
        </div>
    `;
    document.getElementById('s1361-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gross_revenue = Number(fd.get('gross_revenue')) || 0;
        state.net_profit_before_owner_comp = Number(fd.get('net_profit_before_owner_comp')) || 0;
        state.owner_w2_wages_planned = Number(fd.get('owner_w2_wages_planned')) || 0;
        state.industry = fd.get('industry');
        state.market_rate_compensation = Number(fd.get('market_rate_compensation')) || 0;
        state.federal_marginal_rate = Number(fd.get('federal_marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1361-output');
    if (!el) return;
    const distributableProfit = state.net_profit_before_owner_comp - state.owner_w2_wages_planned;
    // SE tax saving = SE tax that WOULD have been owed on profit, less FICA on wages
    const ssWageBase2024 = 168_600;
    const employerHalf = 0.0765;
    const fica = Math.min(state.owner_w2_wages_planned, ssWageBase2024) * employerHalf * 2
        + Math.max(0, state.owner_w2_wages_planned - ssWageBase2024) * 0.029;
    const seTaxIfAllProfit = Math.min(state.net_profit_before_owner_comp * 0.9235, ssWageBase2024) * 0.153
        + Math.max(0, state.net_profit_before_owner_comp * 0.9235 - ssWageBase2024) * 0.029;
    const seSaved = Math.max(0, seTaxIfAllProfit - fica);
    const profitVsMarket = state.market_rate_compensation > 0
        ? state.owner_w2_wages_planned / state.market_rate_compensation
        : 0;
    const compRatio = state.net_profit_before_owner_comp > 0
        ? state.owner_w2_wages_planned / state.net_profit_before_owner_comp
        : 0;
    let riskLevel, riskCls;
    if (compRatio < 0.20 || (state.market_rate_compensation > 0 && profitVsMarket < 0.60)) {
        riskLevel = t('view.s1361.risk.high');
        riskCls = 'neg';
    } else if (compRatio < 0.40 || (state.market_rate_compensation > 0 && profitVsMarket < 0.85)) {
        riskLevel = t('view.s1361.risk.medium');
        riskCls = 'neg';
    } else {
        riskLevel = t('view.s1361.risk.low');
        riskCls = 'pos';
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1361.h2.result">S-corp election outcome</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s1361.card.se_saved">SE tax saved vs Schedule C</div>
                    <div class="value">$${seSaved.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1361.card.fica">FICA on W-2 wages</div>
                    <div class="value">$${fica.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1361.card.distributable">Distributable profit</div>
                    <div class="value">$${distributableProfit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1361.card.comp_ratio">Comp / profit ratio</div>
                    <div class="value">${(compRatio * 100).toFixed(0)}%</div>
                </div>
                ${state.market_rate_compensation > 0 ? `
                    <div class="card">
                        <div class="label" data-i18n="view.s1361.card.market_ratio">Comp / market</div>
                        <div class="value">${(profitVsMarket * 100).toFixed(0)}%</div>
                    </div>
                ` : ''}
                <div class="card ${riskCls}">
                    <div class="label" data-i18n="view.s1361.card.risk">Audit risk</div>
                    <div class="value">${esc(riskLevel)}</div>
                </div>
            </div>
        </div>
    `;
}
