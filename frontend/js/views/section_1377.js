// IRC § 1377 — S-Corp Allocation Elections + Post-Termination Transition Period.
// § 1377(a)(2) "Closing-of-Books" Election: avoid pro-rata allocation when shareholder terminates.
// Requires CONSENT of ALL shareholders + the new / departing shareholder + corp.
// § 1377(b) PTTP (Post-Termination Transition Period): 1 year + 120 days to claim distributions vs AAA.
// § 1377(a)(1) general rule: pro-rata daily allocation across pre / post change shareholders.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    total_year_income: 0,
    days_in_year: 365,
    days_before_change: 0,
    days_after_change: 0,
    departing_shareholder_pct: 0,
    new_shareholder_pct: 0,
    pre_change_actual_income: 0,
    post_change_actual_income: 0,
    closing_books_election: false,
    all_shareholders_consent: false,
    pttp_distribution: 0,
    aaa_at_termination: 0,
    is_s_corp_terminated: false,
    election_year: 2024,
};

export async function renderSection1377(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1377.h1.title">// § 1377 S-CORP ALLOC + PTTP</span></h1>
        <p class="muted small" data-i18n="view.s1377.hint.intro">
            <strong>§ 1377(a)(1) DEFAULT:</strong> pro-rata DAILY allocation when shareholder changes mid-year.
            <strong>§ 1377(a)(2) Closing-of-Books Election:</strong> avoid pro-rata — allocate based on
            ACTUAL income in pre / post-change periods. <strong>Requires CONSENT of ALL affected shareholders
            + corp.</strong> Useful when income lumpy / heavy in one period. <strong>§ 1377(b) PTTP</strong>
            (Post-Termination Transition Period): <strong>1 year + 120 days</strong> after S election terminates
            to distribute AAA tax-free. <strong>§ 1377(a)(2) v. § 1362(e)(3):</strong> different elections —
            term vs change in shareholder.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1377.h2.inputs">Inputs</h2>
            <form id="s1377-form" class="inline-form">
                <label><span data-i18n="view.s1377.label.income">Total year income ($)</span>
                    <input type="number" step="0.01" name="total_year_income" value="${state.total_year_income}"></label>
                <label><span data-i18n="view.s1377.label.days">Days in year</span>
                    <input type="number" step="1" name="days_in_year" value="${state.days_in_year}"></label>
                <label><span data-i18n="view.s1377.label.before">Days before change</span>
                    <input type="number" step="1" name="days_before_change" value="${state.days_before_change}"></label>
                <label><span data-i18n="view.s1377.label.after">Days after change</span>
                    <input type="number" step="1" name="days_after_change" value="${state.days_after_change}"></label>
                <label><span data-i18n="view.s1377.label.departing">Departing shareholder %</span>
                    <input type="number" step="0.1" name="departing_shareholder_pct" value="${state.departing_shareholder_pct}"></label>
                <label><span data-i18n="view.s1377.label.new">New shareholder %</span>
                    <input type="number" step="0.1" name="new_shareholder_pct" value="${state.new_shareholder_pct}"></label>
                <label><span data-i18n="view.s1377.label.pre_actual">Actual pre-change income ($)</span>
                    <input type="number" step="0.01" name="pre_change_actual_income" value="${state.pre_change_actual_income}"></label>
                <label><span data-i18n="view.s1377.label.post_actual">Actual post-change income ($)</span>
                    <input type="number" step="0.01" name="post_change_actual_income" value="${state.post_change_actual_income}"></label>
                <label><span data-i18n="view.s1377.label.election">§ 1377(a)(2) closing-books election?</span>
                    <input type="checkbox" name="closing_books_election" ${state.closing_books_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1377.label.consent">All affected shareholders consent?</span>
                    <input type="checkbox" name="all_shareholders_consent" ${state.all_shareholders_consent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1377.label.pttp_dist">PTTP distribution ($)</span>
                    <input type="number" step="0.01" name="pttp_distribution" value="${state.pttp_distribution}"></label>
                <label><span data-i18n="view.s1377.label.aaa">AAA at termination ($)</span>
                    <input type="number" step="0.01" name="aaa_at_termination" value="${state.aaa_at_termination}"></label>
                <label><span data-i18n="view.s1377.label.terminated">S election terminated?</span>
                    <input type="checkbox" name="is_s_corp_terminated" ${state.is_s_corp_terminated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1377.label.year">Election year</span>
                    <input type="number" step="1" name="election_year" value="${state.election_year}"></label>
                <button class="primary" type="submit" data-i18n="view.s1377.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1377-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1377.h2.allocation">Allocation methods comparison</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1377.th.method">Method</th>
                    <th data-i18n="view.s1377.th.allocation">Allocation</th>
                    <th data-i18n="view.s1377.th.use_case">When best</th>
                </tr></thead>
                <tbody>
                    <tr><td>Pro-rata daily (§ 1377(a)(1) default)</td><td>Daily share × ownership</td><td>Smooth income, no consent needed</td></tr>
                    <tr><td>§ 1377(a)(2) Closing-of-Books</td><td>Actual income pre / post</td><td>Lumpy income, consent of all</td></tr>
                    <tr><td>§ 1362(e)(3) Termination of S election</td><td>Allocate to S period vs C period</td><td>S election terminates mid-year</td></tr>
                    <tr><td>§ 1377(b) PTTP distribution</td><td>Special 1 yr + 120 day window</td><td>Distribute AAA post-termination</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1377.h2.election_details">Closing-books election details (§ 1377(a)(2))</h2>
            <ul class="muted small">
                <li data-i18n="view.s1377.ed.consent">CONSENT: ALL affected shareholders (departing + new + remaining) + corp</li>
                <li data-i18n="view.s1377.ed.statement">Election statement attached to Form 1120-S</li>
                <li data-i18n="view.s1377.ed.timing">Made on Form 1120-S for year of change</li>
                <li data-i18n="view.s1377.ed.irrevocable">Once made for a year: irrevocable for that year</li>
                <li data-i18n="view.s1377.ed.partial">Can be made for specific change events, not all</li>
                <li data-i18n="view.s1377.ed.use_case_loss">Use case: heavy losses in first half (new partner takes hit; departing avoids)</li>
                <li data-i18n="view.s1377.ed.use_case_gain">Use case: heavy gain in second half (new partner reaps; departing avoids tax)</li>
                <li data-i18n="view.s1377.ed.flip_disadvantage">Disadvantage flips: useful for one party, harmful for other — consent reflects negotiation</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1377.h2.pttp">Post-Termination Transition Period (§ 1377(b))</h2>
            <ul class="muted small">
                <li data-i18n="view.s1377.pttp.window">Window: 1 YEAR after termination + 120 days for late challenges</li>
                <li data-i18n="view.s1377.pttp.distributions">Distributions during PTTP treated as AAA / PTI / OAA — tax-free to extent</li>
                <li data-i18n="view.s1377.pttp.s1368_ordering">Standard § 1368 ordering: AAA → PTI → OAA → basis → cap gain</li>
                <li data-i18n="view.s1377.pttp.purpose">Purpose: allow former S-corp shareholders to cash out without dividend tax</li>
                <li data-i18n="view.s1377.pttp.no_extension">Cannot extend PTTP beyond statutory window</li>
                <li data-i18n="view.s1377.pttp.aaa_freeze">AAA balance FREEZES at termination — no further income additions</li>
                <li data-i18n="view.s1377.pttp.attributes_after">Post-PTTP distributions: ordinary dividend (subject to qualified div rules)</li>
                <li data-i18n="view.s1377.pttp.elect_out">§ 1371(e)(2) elect out: revoke step-up to FMV (rarely used)</li>
            </ul>
        </div>
    `;
    document.getElementById('s1377-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_year_income = Number(fd.get('total_year_income')) || 0;
        state.days_in_year = Number(fd.get('days_in_year')) || 0;
        state.days_before_change = Number(fd.get('days_before_change')) || 0;
        state.days_after_change = Number(fd.get('days_after_change')) || 0;
        state.departing_shareholder_pct = Number(fd.get('departing_shareholder_pct')) || 0;
        state.new_shareholder_pct = Number(fd.get('new_shareholder_pct')) || 0;
        state.pre_change_actual_income = Number(fd.get('pre_change_actual_income')) || 0;
        state.post_change_actual_income = Number(fd.get('post_change_actual_income')) || 0;
        state.closing_books_election = !!fd.get('closing_books_election');
        state.all_shareholders_consent = !!fd.get('all_shareholders_consent');
        state.pttp_distribution = Number(fd.get('pttp_distribution')) || 0;
        state.aaa_at_termination = Number(fd.get('aaa_at_termination')) || 0;
        state.is_s_corp_terminated = !!fd.get('is_s_corp_terminated');
        state.election_year = Number(fd.get('election_year')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1377-output');
    if (!el) return;
    const dailyShare = state.days_in_year > 0 ? state.total_year_income / state.days_in_year : 0;
    const prorata_departing = dailyShare * state.days_before_change * (state.departing_shareholder_pct / 100);
    const prorata_new = dailyShare * state.days_after_change * (state.new_shareholder_pct / 100);
    const election_valid = state.closing_books_election && state.all_shareholders_consent;
    const closing_books_departing = election_valid ? state.pre_change_actual_income * (state.departing_shareholder_pct / 100) : prorata_departing;
    const closing_books_new = election_valid ? state.post_change_actual_income * (state.new_shareholder_pct / 100) : prorata_new;
    const pttp_tax_free = state.is_s_corp_terminated ? Math.min(state.pttp_distribution, state.aaa_at_termination) : 0;
    const pttp_taxable = Math.max(0, state.pttp_distribution - pttp_tax_free);
    const pttp_tax = pttp_taxable * 0.20;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1377.h2.result">§ 1377 allocation + PTTP</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1377.card.daily">Daily share</div>
                    <div class="value">$${dailyShare.toFixed(2)}</div>
                </div>
                <div class="card ${election_valid ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s1377.card.election">Election valid?</div>
                    <div class="value">${election_valid ? esc(t('view.s1377.status.yes')) : esc(t('view.s1377.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1377.card.pro_dep">Pro-rata: Departing</div>
                    <div class="value">$${prorata_departing.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1377.card.cb_dep">Closing books: Departing</div>
                    <div class="value">$${closing_books_departing.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1377.card.pro_new">Pro-rata: New</div>
                    <div class="value">$${prorata_new.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1377.card.cb_new">Closing books: New</div>
                    <div class="value">$${closing_books_new.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1377.card.pttp_tf">PTTP tax-free</div>
                    <div class="value">$${pttp_tax_free.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1377.card.pttp_tax">PTTP taxable + 20% tax</div>
                    <div class="value">$${pttp_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${election_valid && state.pre_change_actual_income !== state.post_change_actual_income ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s1377.election_note">
                    Closing-books election reflects ACTUAL income pattern instead of arbitrary daily pro-rata.
                    Valuable when income lumpy — e.g., legal practice w/ year-end fee receipts; construction
                    contract billed at completion; SaaS w/ annual billing. Consent of all shareholders required
                    — negotiation often results in side payments to compensate disadvantaged party.
                </p>
            ` : ''}
        </div>
    `;
}
