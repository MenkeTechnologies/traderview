// IRC § 401(k)(2)(B)(i)(IV) — Hardship Withdrawal.
// "Immediate + heavy financial need" + "necessary" + "no other resources" reasonably available.
// Safe-harbor categories: medical, principal residence purchase, post-secondary education,
// preventing eviction/foreclosure, funeral, principal residence damage.
// SECURE 2.0: hardship withdrawals tax-favored + Roth source allowed + $1k emergency annual.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const TEN_PCT_PENALTY = 0.10;
const EMERGENCY_ANNUAL_2024 = 1_000;

let state = {
    hardship_category: 'medical',
    withdrawal_amount: 0,
    your_age: 35,
    is_roth_source: false,
    is_qualified_birth_adoption: false,
    is_qualified_disaster: false,
    is_terminal_illness: false,
    has_employer_match_pre_2019: true,
    fed_marginal_rate: 0.32,
    state_marginal_rate: 0.06,
};

export async function renderSection401kHardship(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s401h.h1.title">// § 401(k) HARDSHIP WITHDRAWAL</span></h1>
        <p class="muted small" data-i18n="view.s401h.hint.intro">
            "Immediate + heavy financial need" + "necessary" + "no other resources". Safe-harbor
            categories: medical, principal residence purchase, post-secondary education, prevent
            eviction / foreclosure, funeral, principal residence damage. <strong>SECURE 2.0
            improvements:</strong> Roth source allowed + $1k emergency / yr + terminal illness +
            domestic abuse + disaster relief + employer match + own contributions all eligible.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s401h.h2.inputs">Inputs</h2>
            <form id="s401h-form" class="inline-form">
                <label><span data-i18n="view.s401h.label.category">Hardship category</span>
                    <select name="hardship_category">
                        <option value="medical" ${state.hardship_category === 'medical' ? 'selected' : ''}>Medical expenses</option>
                        <option value="home_purchase" ${state.hardship_category === 'home_purchase' ? 'selected' : ''}>Principal residence purchase</option>
                        <option value="education" ${state.hardship_category === 'education' ? 'selected' : ''}>Post-secondary education</option>
                        <option value="eviction" ${state.hardship_category === 'eviction' ? 'selected' : ''}>Prevent eviction / foreclosure</option>
                        <option value="funeral" ${state.hardship_category === 'funeral' ? 'selected' : ''}>Funeral expenses</option>
                        <option value="repair" ${state.hardship_category === 'repair' ? 'selected' : ''}>Principal residence repair (casualty)</option>
                        <option value="disaster" ${state.hardship_category === 'disaster' ? 'selected' : ''}>FEMA-declared disaster (SECURE 2.0)</option>
                        <option value="domestic_abuse" ${state.hardship_category === 'domestic_abuse' ? 'selected' : ''}>Domestic abuse (SECURE 2.0)</option>
                        <option value="emergency_1k" ${state.hardship_category === 'emergency_1k' ? 'selected' : ''}>$1k emergency (SECURE 2.0)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s401h.label.amount">Withdrawal amount ($)</span>
                    <input type="number" step="100" name="withdrawal_amount" value="${state.withdrawal_amount}"></label>
                <label><span data-i18n="view.s401h.label.age">Your age</span>
                    <input type="number" step="1" name="your_age" value="${state.your_age}"></label>
                <label><span data-i18n="view.s401h.label.roth">Roth source?</span>
                    <input type="checkbox" name="is_roth_source" ${state.is_roth_source ? 'checked' : ''}></label>
                <label><span data-i18n="view.s401h.label.birth_adopt">Qualified birth / adoption?</span>
                    <input type="checkbox" name="is_qualified_birth_adoption" ${state.is_qualified_birth_adoption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s401h.label.disaster">FEMA disaster?</span>
                    <input type="checkbox" name="is_qualified_disaster" ${state.is_qualified_disaster ? 'checked' : ''}></label>
                <label><span data-i18n="view.s401h.label.terminal">Terminal illness?</span>
                    <input type="checkbox" name="is_terminal_illness" ${state.is_terminal_illness ? 'checked' : ''}></label>
                <label><span data-i18n="view.s401h.label.pre_2019">Has pre-2019 employer match?</span>
                    <input type="checkbox" name="has_employer_match_pre_2019" ${state.has_employer_match_pre_2019 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s401h.label.fed_rate">Federal marginal %</span>
                    <input type="number" step="0.01" name="fed_marginal_rate" value="${state.fed_marginal_rate}"></label>
                <label><span data-i18n="view.s401h.label.state_rate">State marginal %</span>
                    <input type="number" step="0.01" name="state_marginal_rate" value="${state.state_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s401h.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s401h-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s401h.h2.exceptions">Exceptions to 10% early-withdrawal penalty</h2>
            <ul class="muted small">
                <li data-i18n="view.s401h.ex.age_55">Age 55+ + separated from service (Rule of 55)</li>
                <li data-i18n="view.s401h.ex.medical_7_5">Medical expenses &gt; 7.5% AGI (penalty-free portion)</li>
                <li data-i18n="view.s401h.ex.disability">Total + permanent disability</li>
                <li data-i18n="view.s401h.ex.qdro">QDRO distribution to alternate payee</li>
                <li data-i18n="view.s401h.ex.section_72t">§ 72(t) SEPP — substantially equal periodic payments</li>
                <li data-i18n="view.s401h.ex.public_safety">Public safety employees age 50+ + separation (since 2023)</li>
                <li data-i18n="view.s401h.ex.disaster">FEMA disaster — $22k cap (SECURE 2.0)</li>
                <li data-i18n="view.s401h.ex.terminal">Terminal illness (death within 84 months)</li>
                <li data-i18n="view.s401h.ex.domestic_abuse">Domestic abuse victim (lesser of $10k or 50% balance)</li>
                <li data-i18n="view.s401h.ex.birth_adopt">Qualified birth / adoption ($5k per child)</li>
                <li data-i18n="view.s401h.ex.long_term_care">Long-term care insurance (max $2,500/yr, SECURE 2.0)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s401h.h2.documentation">Documentation required</h2>
            <ul class="muted small">
                <li data-i18n="view.s401h.doc.medical">Medical: itemized bills, doctor's letter, insurance EOBs</li>
                <li data-i18n="view.s401h.doc.home">Home purchase: P&S agreement, closing statement</li>
                <li data-i18n="view.s401h.doc.education">Education: tuition bills + enrollment verification</li>
                <li data-i18n="view.s401h.doc.eviction">Eviction: court papers + landlord notice</li>
                <li data-i18n="view.s401h.doc.funeral">Funeral: itemized funeral bills + death certificate</li>
                <li data-i18n="view.s401h.doc.casualty">Casualty: damage assessment + repair estimates</li>
                <li data-i18n="view.s401h.doc.affirmation">Post-2020 SECURE Act: employee self-certification accepted instead of docs</li>
            </ul>
        </div>
    `;
    document.getElementById('s401h-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.hardship_category = fd.get('hardship_category');
        state.withdrawal_amount = Number(fd.get('withdrawal_amount')) || 0;
        state.your_age = Number(fd.get('your_age')) || 35;
        state.is_roth_source = !!fd.get('is_roth_source');
        state.is_qualified_birth_adoption = !!fd.get('is_qualified_birth_adoption');
        state.is_qualified_disaster = !!fd.get('is_qualified_disaster');
        state.is_terminal_illness = !!fd.get('is_terminal_illness');
        state.has_employer_match_pre_2019 = !!fd.get('has_employer_match_pre_2019');
        state.fed_marginal_rate = Number(fd.get('fed_marginal_rate')) || 0.32;
        state.state_marginal_rate = Number(fd.get('state_marginal_rate')) || 0.06;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s401h-output');
    if (!el) return;
    const isPenaltyExempt = state.your_age >= 59.5
        || state.is_qualified_birth_adoption
        || state.is_qualified_disaster
        || state.is_terminal_illness
        || state.hardship_category === 'emergency_1k'
        || state.hardship_category === 'domestic_abuse'
        || state.hardship_category === 'medical';
    const cappedAmount = state.hardship_category === 'emergency_1k'
        ? Math.min(state.withdrawal_amount, EMERGENCY_ANNUAL_2024)
        : state.withdrawal_amount;
    const fedTax = state.is_roth_source ? 0 : cappedAmount * state.fed_marginal_rate;
    const stateTax = state.is_roth_source ? 0 : cappedAmount * state.state_marginal_rate;
    const penalty = isPenaltyExempt ? 0 : cappedAmount * TEN_PCT_PENALTY;
    const totalTax = fedTax + stateTax + penalty;
    const netReceived = cappedAmount - totalTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s401h.h2.result">Hardship outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s401h.card.gross">Gross withdrawal</div>
                    <div class="value">$${cappedAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s401h.card.fed_tax">Federal income tax</div>
                    <div class="value">$${fedTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s401h.card.state_tax">State income tax</div>
                    <div class="value">$${stateTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${penalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s401h.card.penalty">10% early-withdrawal</div>
                    <div class="value">$${penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${isPenaltyExempt ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s401h.card.exempt">Penalty exempt?</div>
                    <div class="value">${isPenaltyExempt ? esc(t('view.s401h.status.yes')) : esc(t('view.s401h.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s401h.card.total_tax">Total tax cost</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s401h.card.net">Net received</div>
                    <div class="value">$${netReceived.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
