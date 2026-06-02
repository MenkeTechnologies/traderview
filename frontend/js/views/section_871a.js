// IRC § 871(a) — NRA Withholding on Fixed/Determinable/Annual/Periodical (FDAP) Income.
// 30% flat tax on US-source FDAP — withholding agent must remit (Forms W-8BEN + 1042-S).
// FDAP: interest, dividends, rents, royalties, salaries, etc. — passive non-ECI.
// Reduced rate / exemption via TREATY (Form W-8BEN, applicable LOB / treaty article).
// § 871(m) for dividend equivalents from total return swaps.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    income_type: 'dividend',
    gross_amount: 0,
    is_nonresident_alien: true,
    treaty_country: '',
    treaty_rate_pct: 30,
    has_w8ben: true,
    is_us_source: true,
    is_eci_active: false,
    portfolio_interest_exemption: false,
    bank_interest_exemption: false,
    investment_partnership: false,
    treaty_lob_test_met: false,
    treaty_position_disclosed: false,
    qualified_intermediary: false,
};

export async function renderSection871A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s871a.h1.title">// § 871(a) NRA FDAP WITHHOLDING</span></h1>
        <p class="muted small" data-i18n="view.s871a.hint.intro">
            <strong>30% flat tax</strong> on US-source FDAP income to NRAs. Withholding agent must REMIT —
            Forms <strong>W-8BEN</strong> (NRA claim of treaty / status) + <strong>1042-S</strong>
            (information return + 1042 aggregate). <strong>FDAP:</strong> Fixed/Determinable/Annual/Periodical
            — interest, dividends, rents, royalties, salaries (passive non-ECI). <strong>Treaty rates:</strong>
            often 0-15% (LOB + active trade tests). <strong>Exemptions:</strong> portfolio interest § 871(h),
            bank deposit interest § 871(i), capital gains (NRA-domiciled). <strong>§ 871(m):</strong> dividend
            equivalents from total return swaps. <strong>§ 1441 / § 1442 withholding mechanics.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s871a.h2.inputs">Inputs</h2>
            <form id="s871a-form" class="inline-form">
                <label><span data-i18n="view.s871a.label.type">Income type</span>
                    <select name="income_type">
                        <option value="dividend" ${state.income_type === 'dividend' ? 'selected' : ''}>Dividend</option>
                        <option value="interest" ${state.income_type === 'interest' ? 'selected' : ''}>Interest</option>
                        <option value="royalty" ${state.income_type === 'royalty' ? 'selected' : ''}>Royalty</option>
                        <option value="rent" ${state.income_type === 'rent' ? 'selected' : ''}>Rent (US real prop)</option>
                        <option value="services" ${state.income_type === 'services' ? 'selected' : ''}>Personal services</option>
                        <option value="pension" ${state.income_type === 'pension' ? 'selected' : ''}>Pension / retirement</option>
                        <option value="dividend_equivalent" ${state.income_type === 'dividend_equivalent' ? 'selected' : ''}>§ 871(m) dividend equivalent</option>
                        <option value="gambling_winnings" ${state.income_type === 'gambling_winnings' ? 'selected' : ''}>Gambling winnings</option>
                    </select>
                </label>
                <label><span data-i18n="view.s871a.label.amount">Gross amount ($)</span>
                    <input type="number" step="100" name="gross_amount" value="${state.gross_amount}"></label>
                <label><span data-i18n="view.s871a.label.nra">Nonresident alien?</span>
                    <input type="checkbox" name="is_nonresident_alien" ${state.is_nonresident_alien ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871a.label.country">Treaty country</span>
                    <input type="text" name="treaty_country" value="${esc(state.treaty_country)}"></label>
                <label><span data-i18n="view.s871a.label.rate">Treaty rate %</span>
                    <input type="number" step="0.1" name="treaty_rate_pct" value="${state.treaty_rate_pct}"></label>
                <label><span data-i18n="view.s871a.label.w8ben">Form W-8BEN on file?</span>
                    <input type="checkbox" name="has_w8ben" ${state.has_w8ben ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871a.label.us_source">US source income?</span>
                    <input type="checkbox" name="is_us_source" ${state.is_us_source ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871a.label.eci">ECI (active biz)?</span>
                    <input type="checkbox" name="is_eci_active" ${state.is_eci_active ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871a.label.portfolio">§ 871(h) portfolio interest exemption?</span>
                    <input type="checkbox" name="portfolio_interest_exemption" ${state.portfolio_interest_exemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871a.label.bank">§ 871(i) bank deposit interest exemption?</span>
                    <input type="checkbox" name="bank_interest_exemption" ${state.bank_interest_exemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871a.label.partnership">Investment partnership?</span>
                    <input type="checkbox" name="investment_partnership" ${state.investment_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871a.label.lob">Treaty LOB test met?</span>
                    <input type="checkbox" name="treaty_lob_test_met" ${state.treaty_lob_test_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871a.label.disclosed">Treaty position disclosed (Form 8833)?</span>
                    <input type="checkbox" name="treaty_position_disclosed" ${state.treaty_position_disclosed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871a.label.qi">Qualified Intermediary (QI)?</span>
                    <input type="checkbox" name="qualified_intermediary" ${state.qualified_intermediary ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s871a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s871a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871a.h2.fdap_def">FDAP income categories</h2>
            <ul class="muted small">
                <li data-i18n="view.s871a.fdap.dividends">Dividends from US corps (§ 861(a)(2)(A) sourced)</li>
                <li data-i18n="view.s871a.fdap.interest">Interest from US obligors (§ 861(a)(1) sourced)</li>
                <li data-i18n="view.s871a.fdap.royalties">Royalties for US-source IP usage</li>
                <li data-i18n="view.s871a.fdap.rent_personal">Rent from personal property in US</li>
                <li data-i18n="view.s871a.fdap.compensation">Compensation for personal services performed in US</li>
                <li data-i18n="view.s871a.fdap.pensions">Pensions / annuities from US plans</li>
                <li data-i18n="view.s871a.fdap.gambling">Gambling winnings (mostly 30% — exempt: blackjack, baccarat, craps, roulette, big-6 in casinos)</li>
                <li data-i18n="view.s871a.fdap.original_issue">OID accruals + capital gain dividends from RIC / REIT</li>
                <li data-i18n="view.s871a.fdap.scholarships">Scholarships in excess of qualified expenses</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871a.h2.exemptions">Statutory exemptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s871a.exem.portfolio">§ 871(h) portfolio interest: registered debt + foreign payee + non-10% shareholder of issuer</li>
                <li data-i18n="view.s871a.exem.bank_dep">§ 871(i) bank deposit interest: US bank deposits to foreign payee (since 2013)</li>
                <li data-i18n="view.s871a.exem.cap_gains">Capital gains: NRAs generally NOT taxed except (1) US real prop (§ 897 FIRPTA), (2) ECI, (3) 183-day rule for individuals</li>
                <li data-i18n="view.s871a.exem.original_issue_short">Original Issue Discount on short-term obligations (≤ 183 days)</li>
                <li data-i18n="view.s871a.exem.foreign_govt">Foreign government / international org income (§ 892)</li>
                <li data-i18n="view.s871a.exem.casino">Certain casino gambling: blackjack, baccarat, craps, roulette, big-6 wheel</li>
                <li data-i18n="view.s871a.exem.consular">Consular / diplomatic personnel</li>
                <li data-i18n="view.s871a.exem.acquisition_cost">Acquisition (re-)imbursements by US company excluded</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871a.h2.treaty_rates">Common treaty rates (vs 30% default)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s871a.th.country">Country</th>
                    <th data-i18n="view.s871a.th.dividend">Dividend</th>
                    <th data-i18n="view.s871a.th.interest">Interest</th>
                    <th data-i18n="view.s871a.th.royalty">Royalty</th>
                </tr></thead>
                <tbody>
                    <tr><td>UK</td><td>0% / 5% / 15%</td><td>0%</td><td>0%</td></tr>
                    <tr><td>Canada</td><td>5% / 15%</td><td>0%</td><td>0% / 10%</td></tr>
                    <tr><td>Germany</td><td>0% / 5% / 15%</td><td>0%</td><td>0%</td></tr>
                    <tr><td>Japan</td><td>0% / 5% / 10%</td><td>0% / 10%</td><td>0%</td></tr>
                    <tr><td>Switzerland</td><td>5% / 15%</td><td>0% / 5%</td><td>0%</td></tr>
                    <tr><td>Ireland</td><td>5% / 15%</td><td>0%</td><td>0%</td></tr>
                    <tr><td>France</td><td>5% / 15%</td><td>0%</td><td>0%</td></tr>
                    <tr><td>Mexico</td><td>5% / 10% / 15%</td><td>4.9% / 10%</td><td>10%</td></tr>
                    <tr><td>Russia (suspended 2024)</td><td>5% / 10%</td><td>0%</td><td>0%</td></tr>
                    <tr><td>China</td><td>10%</td><td>10%</td><td>10%</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871a.h2.withholding_agents">Withholding agent obligations</h2>
            <ul class="muted small">
                <li data-i18n="view.s871a.wa.1441_1442">§ 1441 individual / § 1442 corporate withholding (parallel)</li>
                <li data-i18n="view.s871a.wa.w8ben">Collect Form W-8BEN (individual) / W-8BEN-E (entity) / W-8ECI (ECI) / W-8IMY (intermediary)</li>
                <li data-i18n="view.s871a.wa.documentation">Document treaty claim before reduced rate applied</li>
                <li data-i18n="view.s871a.wa.expiration">W-8BEN valid 3 yrs from execution + indefinitely if facts unchanged</li>
                <li data-i18n="view.s871a.wa.remit">Remit withholding to IRS (Forms 1042 + 1042-S + 8804)</li>
                <li data-i18n="view.s871a.wa.fatca">FATCA Chapter 4: 30% withholding on payments to non-FATCA compliant foreign entities</li>
                <li data-i18n="view.s871a.wa.penalty">Penalty: 100% of withholding amount + interest if failure</li>
                <li data-i18n="view.s871a.wa.qi">Qualified Intermediary status: streamlines documentation</li>
            </ul>
        </div>
    `;
    document.getElementById('s871a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.income_type = fd.get('income_type');
        state.gross_amount = Number(fd.get('gross_amount')) || 0;
        state.is_nonresident_alien = !!fd.get('is_nonresident_alien');
        state.treaty_country = fd.get('treaty_country');
        state.treaty_rate_pct = Number(fd.get('treaty_rate_pct')) || 0;
        state.has_w8ben = !!fd.get('has_w8ben');
        state.is_us_source = !!fd.get('is_us_source');
        state.is_eci_active = !!fd.get('is_eci_active');
        state.portfolio_interest_exemption = !!fd.get('portfolio_interest_exemption');
        state.bank_interest_exemption = !!fd.get('bank_interest_exemption');
        state.investment_partnership = !!fd.get('investment_partnership');
        state.treaty_lob_test_met = !!fd.get('treaty_lob_test_met');
        state.treaty_position_disclosed = !!fd.get('treaty_position_disclosed');
        state.qualified_intermediary = !!fd.get('qualified_intermediary');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s871a-output');
    if (!el) return;
    const default_rate = 0.30;
    let applicable_rate = default_rate;
    let basis = 'default';
    if (state.portfolio_interest_exemption && state.income_type === 'interest') { applicable_rate = 0; basis = 'portfolio_int'; }
    else if (state.bank_interest_exemption && state.income_type === 'interest') { applicable_rate = 0; basis = 'bank_int'; }
    else if (state.has_w8ben && state.treaty_lob_test_met && state.treaty_rate_pct < default_rate * 100) {
        applicable_rate = state.treaty_rate_pct / 100;
        basis = 'treaty';
    }
    if (!state.is_us_source) { applicable_rate = 0; basis = 'not_us_source'; }
    if (state.is_eci_active) { applicable_rate = 0; basis = 'eci_handled_separately'; }
    const withholding = applicable_rate * state.gross_amount;
    const net_to_nra = state.gross_amount - withholding;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s871a.h2.result">§ 871(a) withholding computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s871a.card.rate">Applicable rate</div>
                    <div class="value">${(applicable_rate * 100).toFixed(1)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s871a.card.basis">Basis</div>
                    <div class="value">${esc(t('view.s871a.basis.' + basis))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s871a.card.gross">Gross amount</div>
                    <div class="value">$${state.gross_amount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s871a.card.withhold">Withholding</div>
                    <div class="value">$${withholding.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s871a.card.net">Net to NRA</div>
                    <div class="value">$${net_to_nra.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!state.has_w8ben && state.treaty_rate_pct < 30 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s871a.no_w8_note">
                    No W-8BEN on file → DEFAULT 30% withholding applied. NRA must file Form 1040NR + W-8BEN
                    retroactively + claim refund. Withholding agent has no liability if W-8BEN obtained
                    timely. Treaty rate not auto-applied — explicit documentation required.
                </p>
            ` : ''}
        </div>
    `;
}
