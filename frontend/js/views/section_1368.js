// IRC § 1368 — S-Corp Distributions.
// S-corp w/o E&P: distribution reduces basis → return of capital (then capital gain).
// S-corp WITH C-corp E&P: distribution applies AAA first (no E&P) then E&P (taxable dividend) then OAA / basis.
// AAA = Accumulated Adjustments Account (post-S-corp income/loss net of distributions).
// PTI = Previously Taxed Income (pre-1983 carryover). OAA = Other Adjustments Account.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    distribution_amount: 0,
    aaa_balance: 0,
    accumulated_e_and_p: 0,
    pti_balance: 0,
    oaa_balance: 0,
    stock_basis: 0,
    has_c_corp_history: false,
    election_aaa_bypass: false,
    election_treat_dividend: false,
    election_close_books: false,
    ordering: 'standard',
};

export async function renderSection1368(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1368.h1.title">// § 1368 S-CORP DISTRIBUTIONS</span></h1>
        <p class="muted small" data-i18n="view.s1368.hint.intro">
            S-corp <strong>without</strong> C-corp E&P: distribution reduces basis → return of capital, then
            capital gain. S-corp <strong>with</strong> C-corp E&P: <strong>AAA first</strong> (tax-free up to
            basis), <strong>then E&P</strong> (taxable dividend), then <strong>OAA / basis</strong>.
            <strong>AAA</strong> = Accumulated Adjustments Account (post-S-corp income/loss net of distributions).
            <strong>PTI</strong> = Previously Taxed Income (pre-1983 carryover). <strong>OAA</strong> = Other
            Adjustments Account (tax-exempt interest). Schedule M-2 of Form 1120-S.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1368.h2.inputs">Inputs</h2>
            <form id="s1368-form" class="inline-form">
                <label><span data-i18n="view.s1368.label.dist">Distribution amount ($)</span>
                    <input type="number" step="0.01" name="distribution_amount" value="${state.distribution_amount}"></label>
                <label><span data-i18n="view.s1368.label.aaa">AAA balance ($)</span>
                    <input type="number" step="0.01" name="aaa_balance" value="${state.aaa_balance}"></label>
                <label><span data-i18n="view.s1368.label.ep">Accumulated C-corp E&P ($)</span>
                    <input type="number" step="0.01" name="accumulated_e_and_p" value="${state.accumulated_e_and_p}"></label>
                <label><span data-i18n="view.s1368.label.pti">PTI balance ($)</span>
                    <input type="number" step="0.01" name="pti_balance" value="${state.pti_balance}"></label>
                <label><span data-i18n="view.s1368.label.oaa">OAA balance ($)</span>
                    <input type="number" step="0.01" name="oaa_balance" value="${state.oaa_balance}"></label>
                <label><span data-i18n="view.s1368.label.basis">Shareholder stock basis ($)</span>
                    <input type="number" step="0.01" name="stock_basis" value="${state.stock_basis}"></label>
                <label><span data-i18n="view.s1368.label.history">Has C-corp E&P history?</span>
                    <input type="checkbox" name="has_c_corp_history" ${state.has_c_corp_history ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1368.label.bypass">§ 1368(e)(3) AAA bypass election?</span>
                    <input type="checkbox" name="election_aaa_bypass" ${state.election_aaa_bypass ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1368.label.dividend_election">§ 1368(f) treat as dividend election?</span>
                    <input type="checkbox" name="election_treat_dividend" ${state.election_treat_dividend ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1368.label.close_books">§ 1377(a)(2) close books election?</span>
                    <input type="checkbox" name="election_close_books" ${state.election_close_books ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1368.label.ordering">Ordering rule</span>
                    <select name="ordering">
                        <option value="standard" ${state.ordering === 'standard' ? 'selected' : ''}>Standard (AAA → E&P → OAA → basis → CG)</option>
                        <option value="bypass" ${state.ordering === 'bypass' ? 'selected' : ''}>AAA bypass election</option>
                        <option value="pti_first" ${state.ordering === 'pti_first' ? 'selected' : ''}>PTI first (pre-1983)</option>
                    </select>
                </label>
                <button class="primary" type="submit" data-i18n="view.s1368.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1368-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1368.h2.ordering">Standard ordering for S-corp w/ C-corp E&P</h2>
            <ol class="muted small">
                <li data-i18n="view.s1368.order.aaa">AAA: tax-free (reduces basis); excess over AAA → next tier</li>
                <li data-i18n="view.s1368.order.pti">PTI: tax-free (no basis effect — pre-1983 holdover)</li>
                <li data-i18n="view.s1368.order.ep">Accumulated E&P: TAXABLE DIVIDEND at qualified rate</li>
                <li data-i18n="view.s1368.order.oaa">OAA: tax-free (reduces basis)</li>
                <li data-i18n="view.s1368.order.basis">Stock basis: tax-free (reduces basis to zero)</li>
                <li data-i18n="view.s1368.order.cg">Excess: CAPITAL GAIN (LTCG if &gt; 1 yr holding)</li>
                <li data-i18n="view.s1368.order.no_ep">If S-corp has NO C-corp E&P: skip dividend tier — pure basis recovery</li>
                <li data-i18n="view.s1368.order.modify">§ 1368(e)(3) elections modify ordering at corporate level</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1368.h2.aaa_compute">AAA computation (Form 1120-S Sch M-2)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1368.aaa.start">Start: prior-year ending AAA</li>
                <li data-i18n="view.s1368.aaa.add_income">Add: ordinary income + sep stated income + tax-exempt interest INCOME</li>
                <li data-i18n="view.s1368.aaa.subtract_loss">Subtract: ordinary loss + sep stated losses + nondeductible expenses</li>
                <li data-i18n="view.s1368.aaa.distributions_last">Distributions subtracted LAST (reduce AAA, not below zero)</li>
                <li data-i18n="view.s1368.aaa.tax_exempt_oaa">Tax-exempt interest goes to OAA (not AAA) — corrected by 2002 regs</li>
                <li data-i18n="view.s1368.aaa.can_be_negative">AAA can be NEGATIVE from losses (but distribution limited to current AAA)</li>
                <li data-i18n="view.s1368.aaa.net_negative">Net negative AAA distributions skipped — go to E&P tier</li>
                <li data-i18n="view.s1368.aaa.s382_ats">Coordinate with § 382 AAA limitation upon ownership change</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1368.h2.elections">Key elections (§ 1368(e)(3), § 1377)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1368.elect.bypass">§ 1368(e)(3) AAA bypass: distribute from E&P FIRST (use up old C-corp E&P)</li>
                <li data-i18n="view.s1368.elect.treat_div">§ 1368(f) treat S-corp distribution as dividend</li>
                <li data-i18n="view.s1368.elect.close_books">§ 1377(a)(2) close books: avoid blending pre/post-event allocation</li>
                <li data-i18n="view.s1368.elect.terminate">§ 1377(b) post-termination transition period (PTTP): 1 year tax-free</li>
                <li data-i18n="view.s1368.elect.qsst_esbt">QSST / ESBT trust beneficiary elections: affect distribution flow</li>
                <li data-i18n="view.s1368.elect.consent">Election requires consent of ALL affected shareholders (Form 1120-S box)</li>
                <li data-i18n="view.s1368.elect.bypass_rationale">AAA bypass valuable if shareholder is C-corp with DRD vs individual w/ QDR</li>
                <li data-i18n="view.s1368.elect.aaa_cleanup">Permanent cleanup: use bypass election to eliminate accumulated C-corp E&P</li>
            </ul>
        </div>
    `;
    document.getElementById('s1368-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.distribution_amount = Number(fd.get('distribution_amount')) || 0;
        state.aaa_balance = Number(fd.get('aaa_balance')) || 0;
        state.accumulated_e_and_p = Number(fd.get('accumulated_e_and_p')) || 0;
        state.pti_balance = Number(fd.get('pti_balance')) || 0;
        state.oaa_balance = Number(fd.get('oaa_balance')) || 0;
        state.stock_basis = Number(fd.get('stock_basis')) || 0;
        state.has_c_corp_history = !!fd.get('has_c_corp_history');
        state.election_aaa_bypass = !!fd.get('election_aaa_bypass');
        state.election_treat_dividend = !!fd.get('election_treat_dividend');
        state.election_close_books = !!fd.get('election_close_books');
        state.ordering = fd.get('ordering');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1368-output');
    if (!el) return;
    let remaining = state.distribution_amount;
    let aaaPortion = 0, ptiPortion = 0, epPortion = 0, oaaPortion = 0, basisPortion = 0, capGainPortion = 0;
    const bypass = state.election_aaa_bypass;
    if (bypass && state.has_c_corp_history) {
        epPortion = Math.min(remaining, state.accumulated_e_and_p);
        remaining -= epPortion;
    }
    aaaPortion = Math.min(remaining, Math.max(0, state.aaa_balance));
    remaining -= aaaPortion;
    ptiPortion = Math.min(remaining, state.pti_balance);
    remaining -= ptiPortion;
    if (!bypass && state.has_c_corp_history) {
        epPortion = Math.min(remaining, state.accumulated_e_and_p);
        remaining -= epPortion;
    }
    oaaPortion = Math.min(remaining, state.oaa_balance);
    remaining -= oaaPortion;
    basisPortion = Math.min(remaining, Math.max(0, state.stock_basis - aaaPortion - oaaPortion));
    remaining -= basisPortion;
    capGainPortion = remaining;
    const dividendTax = epPortion * 0.20;
    const capGainTax = capGainPortion * 0.20;
    const totalTax = dividendTax + capGainTax;
    const taxFreePortion = aaaPortion + ptiPortion + oaaPortion + basisPortion;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1368.h2.result">§ 1368 distribution tiers</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s1368.card.aaa">AAA (tax-free)</div>
                    <div class="value">$${aaaPortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1368.card.pti">PTI (tax-free)</div>
                    <div class="value">$${ptiPortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1368.card.ep">E&P (dividend)</div>
                    <div class="value">$${epPortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1368.card.oaa">OAA (tax-free)</div>
                    <div class="value">$${oaaPortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1368.card.basis">Basis return</div>
                    <div class="value">$${basisPortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1368.card.capgain">Capital gain</div>
                    <div class="value">$${capGainPortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1368.card.tax_free">Total tax-free</div>
                    <div class="value">$${taxFreePortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1368.card.total_tax">Total tax (20%)</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.has_c_corp_history && bypass ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s1368.bypass_note">
                    AAA bypass election in effect: distribute E&P first to permanently eliminate C-corp
                    E&P. Valuable cleanup when shareholders prefer dividend treatment now (e.g., DRD-eligible
                    corp shareholder) over future deferral.
                </p>
            ` : ''}
        </div>
    `;
}
