// IRC § 1035 — Tax-Free Exchange of Insurance / Annuity Contracts.
// Replace annuity with another annuity / LTC contract / qualified LTC rider — no gain.
// Replace life insurance with life / annuity / LTC. Replace LTC with LTC.
// Cannot go BACKWARD: annuity → life insurance NOT allowed.
// Basis carries over. Loss never recognized on § 1035.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    old_contract_type: 'annuity',
    new_contract_type: 'annuity',
    old_cash_value: 0,
    old_basis: 0,
    boot_received: 0,
    loan_extinguished: 0,
    is_mec: false,
    has_partial_exchange: false,
    partial_amount: 0,
    marginal_rate: 0.32,
};

const EXCHANGES_ALLOWED = {
    life: ['life', 'annuity', 'ltc'],
    annuity: ['annuity', 'ltc'],
    ltc: ['ltc'],
    endowment: ['endowment', 'annuity', 'ltc'],
};

export async function renderSection1035(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1035.h1.title">// § 1035 INSURANCE / ANNUITY EXCHANGE</span></h1>
        <p class="muted small" data-i18n="view.s1035.hint.intro">
            Tax-free replacement of insurance / annuity / endowment / LTC contracts.
            <strong>One-way street:</strong> annuity → life insurance NOT allowed; life → annuity OK.
            Basis carries over. Loss NEVER recognized. <strong>Pension Protection Act 2006:</strong>
            partial exchanges allowed but must follow Rev. Proc. 2011-38: 180-day rule + no
            distributions for 180 days from either contract.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1035.h2.inputs">Inputs</h2>
            <form id="s1035-form" class="inline-form">
                <label><span data-i18n="view.s1035.label.old_type">Old contract</span>
                    <select name="old_contract_type">
                        <option value="life" ${state.old_contract_type === 'life' ? 'selected' : ''}>Life insurance</option>
                        <option value="annuity" ${state.old_contract_type === 'annuity' ? 'selected' : ''}>Annuity</option>
                        <option value="endowment" ${state.old_contract_type === 'endowment' ? 'selected' : ''}>Endowment</option>
                        <option value="ltc" ${state.old_contract_type === 'ltc' ? 'selected' : ''}>LTC</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1035.label.new_type">New contract</span>
                    <select name="new_contract_type">
                        <option value="life" ${state.new_contract_type === 'life' ? 'selected' : ''}>Life insurance</option>
                        <option value="annuity" ${state.new_contract_type === 'annuity' ? 'selected' : ''}>Annuity</option>
                        <option value="ltc" ${state.new_contract_type === 'ltc' ? 'selected' : ''}>LTC</option>
                        <option value="endowment" ${state.new_contract_type === 'endowment' ? 'selected' : ''}>Endowment</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1035.label.cash_value">Old contract cash value ($)</span>
                    <input type="number" step="1000" name="old_cash_value" value="${state.old_cash_value}"></label>
                <label><span data-i18n="view.s1035.label.basis">Old contract basis ($)</span>
                    <input type="number" step="1000" name="old_basis" value="${state.old_basis}"></label>
                <label><span data-i18n="view.s1035.label.boot">Boot received ($)</span>
                    <input type="number" step="100" name="boot_received" value="${state.boot_received}"></label>
                <label><span data-i18n="view.s1035.label.loan">Outstanding loan extinguished ($)</span>
                    <input type="number" step="100" name="loan_extinguished" value="${state.loan_extinguished}"></label>
                <label><span data-i18n="view.s1035.label.mec">Old contract is MEC?</span>
                    <input type="checkbox" name="is_mec" ${state.is_mec ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1035.label.partial">Partial exchange?</span>
                    <input type="checkbox" name="has_partial_exchange" ${state.has_partial_exchange ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1035.label.partial_amount">Partial amount ($)</span>
                    <input type="number" step="1000" name="partial_amount" value="${state.partial_amount}"></label>
                <label><span data-i18n="view.s1035.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s1035.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1035-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1035.h2.exchanges">Allowed exchanges (one-way)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1035.th.old">Old contract</th>
                    <th data-i18n="view.s1035.th.allowed">Can exchange to</th>
                </tr></thead>
                <tbody>
                    <tr><td>Life insurance</td><td>Life / Annuity / LTC / Endowment</td></tr>
                    <tr><td>Annuity</td><td>Annuity / LTC ONLY</td></tr>
                    <tr><td>Endowment</td><td>Annuity / LTC / Endowment of same / lesser maturity</td></tr>
                    <tr><td>LTC</td><td>LTC ONLY</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1035.h2.boot">Boot trap + loan extinguishment</h2>
            <ul class="muted small">
                <li data-i18n="view.s1035.boot.cash">Cash received = boot, gain to extent of gain</li>
                <li data-i18n="view.s1035.boot.loan">Loan extinguished by new carrier = boot (Estate of Halby; Greene)</li>
                <li data-i18n="view.s1035.boot.partial">Partial exchange + distribution within 180 days = taxable</li>
                <li data-i18n="view.s1035.boot.mec_carry">MEC contract: new contract IS ALSO MEC (no laundering)</li>
                <li data-i18n="view.s1035.boot.owner_assignee">Owner / assignee / insured must match (per Rev. Proc. 92-44)</li>
                <li data-i18n="view.s1035.boot.long_term">Holding period TACKS</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1035.h2.uses">Common § 1035 uses</h2>
            <ul class="muted small">
                <li data-i18n="view.s1035.use.cheaper">Move VA to lower-cost VA (M&E + admin saves $30-100k over 20 yrs)</li>
                <li data-i18n="view.s1035.use.no_lockout">Move past surrender-charge period to fresh investment options</li>
                <li data-i18n="view.s1035.use.guaranteed">Move to better income / death benefit / GMxB rider</li>
                <li data-i18n="view.s1035.use.term_to_whole">Convert term life to whole life</li>
                <li data-i18n="view.s1035.use.ltc">Move outdated LTC to modern hybrid LTC-life</li>
                <li data-i18n="view.s1035.use.consolidate">Consolidate multiple policies for simpler estate admin</li>
            </ul>
        </div>
    `;
    document.getElementById('s1035-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.old_contract_type = fd.get('old_contract_type');
        state.new_contract_type = fd.get('new_contract_type');
        state.old_cash_value = Number(fd.get('old_cash_value')) || 0;
        state.old_basis = Number(fd.get('old_basis')) || 0;
        state.boot_received = Number(fd.get('boot_received')) || 0;
        state.loan_extinguished = Number(fd.get('loan_extinguished')) || 0;
        state.is_mec = !!fd.get('is_mec');
        state.has_partial_exchange = !!fd.get('has_partial_exchange');
        state.partial_amount = Number(fd.get('partial_amount')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1035-output');
    if (!el) return;
    const allowed = EXCHANGES_ALLOWED[state.old_contract_type] || [];
    const exchangeAllowed = allowed.includes(state.new_contract_type);
    const embeddedGain = Math.max(0, state.old_cash_value - state.old_basis);
    const totalBoot = state.boot_received + state.loan_extinguished;
    const gainRecognized = exchangeAllowed
        ? Math.min(embeddedGain, totalBoot)
        : embeddedGain;
    const tax = gainRecognized * state.marginal_rate;
    const earlyPenalty = (state.is_mec && tax > 0) ? gainRecognized * 0.10 : 0;
    const newBasis = exchangeAllowed
        ? Math.max(0, state.old_basis - state.boot_received + gainRecognized)
        : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1035.h2.result">§ 1035 outcome</h2>
            <div class="cards">
                <div class="card ${exchangeAllowed ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1035.card.allowed">Exchange allowed?</div>
                    <div class="value">${exchangeAllowed ? esc(t('view.s1035.status.yes')) : esc(t('view.s1035.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1035.card.embedded">Embedded gain</div>
                    <div class="value">$${embeddedGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalBoot > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s1035.card.boot">Total boot</div>
                    <div class="value">$${totalBoot.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${gainRecognized > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1035.card.gain">Gain recognized</div>
                    <div class="value">$${gainRecognized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1035.card.tax">Tax</div>
                    <div class="value">$${tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${earlyPenalty > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s1035.card.penalty">MEC 10% early-withdrawal</div>
                        <div class="value">$${earlyPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card">
                    <div class="label" data-i18n="view.s1035.card.new_basis">New contract basis</div>
                    <div class="value">$${newBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!exchangeAllowed ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1035.warning.disallowed">
                    Exchange NOT allowed under § 1035 — transaction treated as TAXABLE SURRENDER
                    of old contract + purchase of new. Entire gain recognized.
                </p>
            ` : ''}
        </div>
    `;
}
