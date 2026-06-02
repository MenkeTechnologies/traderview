// IRC § 83 — Restricted Property + § 83(b) Election.
// Property received for services taxed at FMV WHEN vested (no substantial risk of forfeiture).
// § 83(b) election: tax at GRANT instead — locks in low FMV, starts capital gain clock.
// Must file within 30 days of grant; irrevocable (except mistake/duress).
// Common: founder stock, restricted stock awards (RSAs), profits interests.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    grant_fmv: 0,
    grant_price_paid: 0,
    vest_fmv: 0,
    sale_price: 0,
    days_since_grant: 0,
    has_substantial_risk: true,
    s83b_election_filed: false,
    holding_period_after_vest_days: 0,
    is_iso_or_nso: 'none',
    is_capital_or_ordinary: 'capital',
    shares_count: 0,
    forfeited_before_vest: false,
};

export async function renderSection83(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s83.h1.title">// § 83 RESTRICTED PROPERTY</span></h1>
        <p class="muted small" data-i18n="view.s83.hint.intro">
            Property received for services taxed at FMV <strong>WHEN VESTED</strong> (substantial risk of
            forfeiture lapses). <strong>§ 83(b) election:</strong> tax at GRANT — locks in low FMV; starts
            capital gain clock. <strong>30-day deadline</strong>; irrevocable. <strong>If forfeit BEFORE vest:</strong>
            no deduction for tax paid (cap loss only if § 83(b)). Common: <strong>founder stock</strong>,
            <strong>restricted stock awards (RSAs)</strong>, <strong>profits interests</strong> (Rev. Proc. 93-27).
            <strong>RSUs:</strong> NOT § 83 (separate § 451 timing).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s83.h2.inputs">Inputs</h2>
            <form id="s83-form" class="inline-form">
                <label><span data-i18n="view.s83.label.grant_fmv">Grant date FMV per share ($)</span>
                    <input type="number" step="0.01" name="grant_fmv" value="${state.grant_fmv}"></label>
                <label><span data-i18n="view.s83.label.grant_price">Price paid per share ($)</span>
                    <input type="number" step="0.01" name="grant_price_paid" value="${state.grant_price_paid}"></label>
                <label><span data-i18n="view.s83.label.vest_fmv">Vest date FMV per share ($)</span>
                    <input type="number" step="0.01" name="vest_fmv" value="${state.vest_fmv}"></label>
                <label><span data-i18n="view.s83.label.sale">Sale price per share ($)</span>
                    <input type="number" step="0.01" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s83.label.days">Days since grant</span>
                    <input type="number" step="1" name="days_since_grant" value="${state.days_since_grant}"></label>
                <label><span data-i18n="view.s83.label.risk">Substantial risk of forfeiture?</span>
                    <input type="checkbox" name="has_substantial_risk" ${state.has_substantial_risk ? 'checked' : ''}></label>
                <label><span data-i18n="view.s83.label.s83b">§ 83(b) election filed?</span>
                    <input type="checkbox" name="s83b_election_filed" ${state.s83b_election_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s83.label.holding">Days held after vest</span>
                    <input type="number" step="1" name="holding_period_after_vest_days" value="${state.holding_period_after_vest_days}"></label>
                <label><span data-i18n="view.s83.label.type">Equity type</span>
                    <select name="is_iso_or_nso">
                        <option value="none" ${state.is_iso_or_nso === 'none' ? 'selected' : ''}>Restricted stock (not option)</option>
                        <option value="iso" ${state.is_iso_or_nso === 'iso' ? 'selected' : ''}>ISO § 422 (special rules)</option>
                        <option value="nso" ${state.is_iso_or_nso === 'nso' ? 'selected' : ''}>NSO § 83 ordinary</option>
                        <option value="profits" ${state.is_iso_or_nso === 'profits' ? 'selected' : ''}>Profits interest (Rev. Proc. 93-27)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s83.label.shares">Number of shares</span>
                    <input type="number" step="1" name="shares_count" value="${state.shares_count}"></label>
                <label><span data-i18n="view.s83.label.forfeit">Forfeited before vest?</span>
                    <input type="checkbox" name="forfeited_before_vest" ${state.forfeited_before_vest ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s83.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s83-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s83.h2.s83b_advantages">§ 83(b) advantages</h2>
            <ul class="muted small">
                <li data-i18n="view.s83.adv.low_fmv">Lock in LOW grant FMV → ordinary income on $0-ish at grant (founder stock)</li>
                <li data-i18n="view.s83.adv.cap_gain">Starts capital gain holding period → LTCG (20%/15%/0%) on appreciation</li>
                <li data-i18n="view.s83.adv.lt_qsbs">Enables § 1202 QSBS 5-yr holding period clock</li>
                <li data-i18n="view.s83.adv.shelter">Shelter all future appreciation as LTCG instead of ordinary on vest</li>
                <li data-i18n="view.s83.adv.amt_avoid">ISO context: avoid AMT preference item on exercise (combined w/ § 422)</li>
                <li data-i18n="view.s83.adv.partnership_capital">Profits interest: receive partnership interest, no ordinary income (Rev. Proc. 93-27 safe harbor)</li>
                <li data-i18n="view.s83.adv.estate_planning">Estate planning: lock in low basis NOW for gift / transfer at low value</li>
                <li data-i18n="view.s83.adv.public_company">Public company: low-vol stock = small risk; high-vol = significant risk if forfeit</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s83.h2.s83b_risks">§ 83(b) risks</h2>
            <ul class="muted small">
                <li data-i18n="view.s83.risk.tax_now">Tax PAID NOW even if stock never vests</li>
                <li data-i18n="view.s83.risk.no_refund">Forfeiture → NO refund of tax paid (sunk cost)</li>
                <li data-i18n="view.s83.risk.capital_loss">Forfeiture → only CAPITAL LOSS (limited $3K/yr against ordinary)</li>
                <li data-i18n="view.s83.risk.cash_flow">Cash flow: need cash to pay tax on illiquid stock</li>
                <li data-i18n="view.s83.risk.high_fmv">High FMV at grant: tax bill may exceed value</li>
                <li data-i18n="view.s83.risk.appraisal">409A appraisal: PRE-FUNDING risk if FMV not defensible</li>
                <li data-i18n="view.s83.risk.30_days">30-DAY deadline: certified mail + return receipt + email copy + accountant</li>
                <li data-i18n="view.s83.risk.late_election">Late election: NO RELIEF except mistake / duress; signed waiver insufficient</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s83.h2.s83b_mechanics">§ 83(b) filing mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.s83.mech.30_days">Within 30 days of GRANT (not vest) — postmark counts</li>
                <li data-i18n="view.s83.mech.statement">File: written statement (IRS sample) — name, SSN, property description, grant date, FMV, restrictions</li>
                <li data-i18n="view.s83.mech.copy_to_employer">Copy to: employer (W-2 reporting) + retain copy + attach to current-year return</li>
                <li data-i18n="view.s83.mech.payment">Pay tax at marginal rate × (FMV − price paid) × shares</li>
                <li data-i18n="view.s83.mech.cash_payment_option">Option: have employer withhold via additional W-2 wages + tax withholding</li>
                <li data-i18n="view.s83.mech.no_email">EMAIL only is NOT sufficient — must be physical mail (certified)</li>
                <li data-i18n="view.s83.mech.2015_reg_change">2015 reg: no longer required to attach to current-year return — but recommended</li>
                <li data-i18n="view.s83.mech.address">IRS service center where you file 1040 (find at irs.gov)</li>
            </ol>
        </div>
    `;
    document.getElementById('s83-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.grant_fmv = Number(fd.get('grant_fmv')) || 0;
        state.grant_price_paid = Number(fd.get('grant_price_paid')) || 0;
        state.vest_fmv = Number(fd.get('vest_fmv')) || 0;
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.days_since_grant = Number(fd.get('days_since_grant')) || 0;
        state.has_substantial_risk = !!fd.get('has_substantial_risk');
        state.s83b_election_filed = !!fd.get('s83b_election_filed');
        state.holding_period_after_vest_days = Number(fd.get('holding_period_after_vest_days')) || 0;
        state.is_iso_or_nso = fd.get('is_iso_or_nso');
        state.shares_count = Number(fd.get('shares_count')) || 0;
        state.forfeited_before_vest = !!fd.get('forfeited_before_vest');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s83-output');
    if (!el) return;
    const grantOrdinaryAmt = (state.grant_fmv - state.grant_price_paid) * state.shares_count;
    const vestOrdinaryAmt = (state.vest_fmv - state.grant_price_paid) * state.shares_count;
    const ordinaryAtGrant = state.s83b_election_filed ? grantOrdinaryAmt : 0;
    const ordinaryAtVest = (!state.s83b_election_filed && !state.forfeited_before_vest) ? vestOrdinaryAmt : 0;
    const basisAfter = state.s83b_election_filed ? state.grant_fmv * state.shares_count : state.vest_fmv * state.shares_count;
    const saleGain = state.sale_price * state.shares_count - basisAfter;
    const isLTCG = state.holding_period_after_vest_days >= 366;
    const capGainRate = isLTCG ? 0.20 : 0.37;
    const ordinaryTax = (ordinaryAtGrant + ordinaryAtVest) * 0.37;
    const capGainTax = saleGain > 0 ? saleGain * capGainRate : saleGain * 0.37;
    const totalTax = ordinaryTax + capGainTax;
    const noElectionTax = vestOrdinaryAmt * 0.37 + Math.max(0, state.sale_price * state.shares_count - state.vest_fmv * state.shares_count) * capGainRate;
    const electionTax = grantOrdinaryAmt * 0.37 + Math.max(0, state.sale_price * state.shares_count - state.grant_fmv * state.shares_count) * capGainRate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s83.h2.result">§ 83 / § 83(b) computation</h2>
            <div class="cards">
                <div class="card ${state.s83b_election_filed ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s83.card.election">§ 83(b) elected?</div>
                    <div class="value">${state.s83b_election_filed ? esc(t('view.s83.status.yes')) : esc(t('view.s83.status.no'))}</div>
                </div>
                <div class="card ${state.days_since_grant > 30 && !state.s83b_election_filed ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s83.card.deadline">30-day deadline</div>
                    <div class="value">${state.days_since_grant > 30 ? esc(t('view.s83.status.past')) : esc(t('view.s83.status.ok'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s83.card.ordinary_grant">Ordinary at grant</div>
                    <div class="value">$${ordinaryAtGrant.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s83.card.ordinary_vest">Ordinary at vest</div>
                    <div class="value">$${ordinaryAtVest.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s83.card.sale_gain">Sale gain / loss</div>
                    <div class="value">$${saleGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${isLTCG ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s83.card.character">Sale character</div>
                    <div class="value">${isLTCG ? esc(t('view.s83.char.ltcg')) : esc(t('view.s83.char.stcg'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s83.card.total_tax">Total tax (current scenario)</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s83.card.compare_savings">Savings vs no election</div>
                    <div class="value">$${(noElectionTax - electionTax).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.forfeited_before_vest && state.s83b_election_filed ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s83.forfeit_note">
                    Forfeited after § 83(b) election: ordinary income still recognized at grant. NO REFUND of
                    tax paid. Only relief: capital loss equal to PRICE PAID (NOT tax basis) — limited to $3K/yr
                    against ordinary. This is the major risk of § 83(b) — confirm vesting probability before electing.
                </p>
            ` : ''}
        </div>
    `;
}
