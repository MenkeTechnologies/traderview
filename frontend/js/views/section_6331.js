// IRC § 6331 — IRS Levy / Wage Garnishment.
// Levy seizes property after notice + demand + Final Notice of Intent + 30 days.
// CDP right under § 6330: 30 days to request hearing → suspends collection.
// Exemptions: tools of trade ($5,540 2024), unemployment benefits, child support, basic living.
// Continuous wage levy: takes everything ABOVE exempt amount (Form 668-W).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const TOOLS_OF_TRADE_2024 = 5_540;
const FUEL_INCOME_2024 = 13_510;
const BASIC_LIVING_EXEMPTION_SINGLE_2024 = 13_850;
const BASIC_LIVING_EXEMPTION_MFJ_2024 = 27_700;

let state = {
    levy_amount: 0,
    gross_wages_monthly: 0,
    filing_status: 'single',
    dependents: 0,
    cdp_hearing_requested: false,
    in_installment_agreement: false,
    has_final_notice: false,
    days_since_final_notice: 0,
    homestead_value: 0,
    bank_account_balance: 0,
    tools_of_trade_value: 0,
    receives_social_security: false,
};

export async function renderSection6331(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6331.h1.title">// § 6331 LEVY / WAGE GARNISHMENT</span></h1>
        <p class="muted small" data-i18n="view.s6331.hint.intro">
            Levy seizes property after <strong>notice + demand + Final Notice of Intent + 30 days</strong>.
            <strong>CDP right under § 6330:</strong> 30 days to request hearing → SUSPENDS collection.
            Exemptions: tools of trade ($5,540 2024), unemployment, child support, basic living
            standard. <strong>Continuous wage levy</strong> takes everything ABOVE exempt amount
            (Form 668-W). Bank levy is ONE-TIME (snapshot at receipt).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6331.h2.inputs">Inputs</h2>
            <form id="s6331-form" class="inline-form">
                <label><span data-i18n="view.s6331.label.levy_amount">Levy amount ($)</span>
                    <input type="number" step="1000" name="levy_amount" value="${state.levy_amount}"></label>
                <label><span data-i18n="view.s6331.label.wages">Gross wages monthly ($)</span>
                    <input type="number" step="100" name="gross_wages_monthly" value="${state.gross_wages_monthly}"></label>
                <label><span data-i18n="view.s6331.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6331.label.dependents">Dependents</span>
                    <input type="number" step="1" name="dependents" value="${state.dependents}"></label>
                <label><span data-i18n="view.s6331.label.cdp">CDP hearing requested?</span>
                    <input type="checkbox" name="cdp_hearing_requested" ${state.cdp_hearing_requested ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6331.label.ia">In installment agreement?</span>
                    <input type="checkbox" name="in_installment_agreement" ${state.in_installment_agreement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6331.label.notice">Has Final Notice (LT11)?</span>
                    <input type="checkbox" name="has_final_notice" ${state.has_final_notice ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6331.label.days_since">Days since Final Notice</span>
                    <input type="number" step="1" name="days_since_final_notice" value="${state.days_since_final_notice}"></label>
                <label><span data-i18n="view.s6331.label.homestead">Homestead value ($)</span>
                    <input type="number" step="10000" name="homestead_value" value="${state.homestead_value}"></label>
                <label><span data-i18n="view.s6331.label.bank">Bank balance ($)</span>
                    <input type="number" step="100" name="bank_account_balance" value="${state.bank_account_balance}"></label>
                <label><span data-i18n="view.s6331.label.tools">Tools of trade value ($)</span>
                    <input type="number" step="100" name="tools_of_trade_value" value="${state.tools_of_trade_value}"></label>
                <label><span data-i18n="view.s6331.label.ss">Receives Social Security?</span>
                    <input type="checkbox" name="receives_social_security" ${state.receives_social_security ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6331.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6331-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6331.h2.exemptions">§ 6334 exempt property</h2>
            <ul class="muted small">
                <li data-i18n="view.s6331.ex.necessary">Necessary clothing + schoolbooks</li>
                <li data-i18n="view.s6331.ex.fuel">$13,510 (2024) of fuel + provisions + furniture + personal effects</li>
                <li data-i18n="view.s6331.ex.tools">$5,540 (2024) of tools / books of trade</li>
                <li data-i18n="view.s6331.ex.unemployment">Unemployment + workers compensation benefits</li>
                <li data-i18n="view.s6331.ex.disability">Workers comp / disability pay</li>
                <li data-i18n="view.s6331.ex.child_support">Court-ordered child support</li>
                <li data-i18n="view.s6331.ex.basic_living">Basic living standard (per IRS exemption tables)</li>
                <li data-i18n="view.s6331.ex.principal_residence">Principal residence requires court order (§ 6334(e))</li>
                <li data-i18n="view.s6331.ex.armed_forces">Armed forces pay (limited)</li>
                <li data-i18n="view.s6331.ex.public_assistance">Public assistance (TANF, food stamps)</li>
                <li data-i18n="view.s6331.ex.social_security">Social Security: only 15% can be levied (FPLP automated levy)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6331.h2.responses">Levy responses</h2>
            <ul class="muted small">
                <li data-i18n="view.s6331.res.cdp">CDP hearing within 30 days of Final Notice (Form 12153)</li>
                <li data-i18n="view.s6331.res.ia">Enter installment agreement (suspends collection)</li>
                <li data-i18n="view.s6331.res.oic">Offer-in-Compromise (Form 656)</li>
                <li data-i18n="view.s6331.res.cnc">Currently Not Collectible status (financial hardship)</li>
                <li data-i18n="view.s6331.res.bankruptcy">Bankruptcy: automatic stay halts collection</li>
                <li data-i18n="view.s6331.res.spouse">Innocent spouse relief Form 8857</li>
                <li data-i18n="view.s6331.res.release">Request levy release: Form 911 or Taxpayer Advocate</li>
                <li data-i18n="view.s6331.res.equivalent">Equivalent hearing (post-30-day) Form 13753 — less powerful but available</li>
            </ul>
        </div>
    `;
    document.getElementById('s6331-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.levy_amount = Number(fd.get('levy_amount')) || 0;
        state.gross_wages_monthly = Number(fd.get('gross_wages_monthly')) || 0;
        state.filing_status = fd.get('filing_status');
        state.dependents = Number(fd.get('dependents')) || 0;
        state.cdp_hearing_requested = !!fd.get('cdp_hearing_requested');
        state.in_installment_agreement = !!fd.get('in_installment_agreement');
        state.has_final_notice = !!fd.get('has_final_notice');
        state.days_since_final_notice = Number(fd.get('days_since_final_notice')) || 0;
        state.homestead_value = Number(fd.get('homestead_value')) || 0;
        state.bank_account_balance = Number(fd.get('bank_account_balance')) || 0;
        state.tools_of_trade_value = Number(fd.get('tools_of_trade_value')) || 0;
        state.receives_social_security = !!fd.get('receives_social_security');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6331-output');
    if (!el) return;
    const wageExemption = state.filing_status === 'mfj' ? BASIC_LIVING_EXEMPTION_MFJ_2024 : BASIC_LIVING_EXEMPTION_SINGLE_2024;
    const dependentBonus = state.dependents * 5_050;
    const monthlyExempt = (wageExemption + dependentBonus) / 12;
    const wageLevyMonthly = Math.max(0, state.gross_wages_monthly - monthlyExempt);
    const toolsExempt = Math.min(state.tools_of_trade_value, TOOLS_OF_TRADE_2024);
    const bankLevyable = state.bank_account_balance;
    const homesteadExempt = state.homestead_value;  // requires court order
    const cdpHalts = state.cdp_hearing_requested && state.days_since_final_notice <= 30;
    const iaProtects = state.in_installment_agreement;
    const collectionBlocked = cdpHalts || iaProtects;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6331.h2.result">Levy outcome</h2>
            <div class="cards">
                <div class="card ${collectionBlocked ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6331.card.blocked">Collection currently blocked?</div>
                    <div class="value">${collectionBlocked ? esc(t('view.s6331.status.yes')) : esc(t('view.s6331.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6331.card.monthly_exempt">Monthly exempt amount</div>
                    <div class="value">$${monthlyExempt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6331.card.wage_levy">Wage levy / mo</div>
                    <div class="value">$${wageLevyMonthly.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6331.card.bank">Bank balance (one-time levy)</div>
                    <div class="value">$${bankLevyable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6331.card.tools_exempt">Tools exempt</div>
                    <div class="value">$${toolsExempt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6331.card.homestead_exempt">Homestead (court order required)</div>
                    <div class="value">$${homesteadExempt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.receives_social_security ? `
                    <div class="card">
                        <div class="label" data-i18n="view.s6331.card.ss_levy">SS levy max (FPLP 15%)</div>
                        <div class="value">15%</div>
                    </div>
                ` : ''}
            </div>
            ${!collectionBlocked && state.has_final_notice && state.days_since_final_notice <= 30 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6331.warning.cdp_urgent">
                    URGENT: Final Notice (LT11) received. File Form 12153 within 30 days to request
                    CDP hearing — preserves all collection alternatives + Tax Court review of any
                    determination. Missing window means equivalent hearing only.
                </p>
            ` : ''}
        </div>
    `;
}
