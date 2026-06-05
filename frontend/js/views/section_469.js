// IRC § 469 — Passive Activity Loss limits + Real Estate Professional test.
// Passive losses only offset passive income. Suspended PALs carry indefinitely
// until disposition or sufficient passive income. REP status (≥750 hrs/yr +
// > 50% of personal services in real estate) unlocks unlimited rental loss
// deduction against ordinary. Material participation tests (Reg § 1.469-5T).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    // Activity counts
    rental_loss: 0,
    rental_income: 0,
    other_passive_income: 0,
    suspended_pal_carryover: 0,
    // REP qualification
    hours_in_rentals: 0,
    hours_in_other_work: 0,
    is_spouse_rep: false,
    spouse_hours_in_rentals: 0,
    // Material participation flags
    mp_more_than_500: false,
    mp_only_one_doing_most: false,
    mp_100_and_no_one_more: false,
    mp_facts_circumstances: false,
    // Self-rental
    self_rental_to_own_biz: false,
    marginal_rate: 0.32,
};

export async function renderSection469(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s469.h1.title">// § 469 PASSIVE ACTIVITY + REP</span></h1>
        <p class="muted small" data-i18n="view.s469.hint.intro">
            Passive losses only offset passive income — excess <strong>SUSPENDED</strong>
            indefinitely until activity disposition or future passive income absorbs.
            <strong>Real Estate Professional</strong> (REP): ≥ 750 hrs/yr in real estate trades
            AND > 50% of personal services in real estate. REP rentals lose passive presumption
            → potentially unlimited offset against ordinary income (still need material
            participation per activity, or aggregation election).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s469.h2.activity_inputs">Activity inputs</h2>
            <form id="s469-form" class="inline-form">
                <label><span data-i18n="view.s469.label.rental_loss">Current-year rental loss ($)</span>
                    <input type="number" step="0.01" name="rental_loss" value="${state.rental_loss}"></label>
                <label><span data-i18n="view.s469.label.rental_income">Current-year rental income ($)</span>
                    <input type="number" step="0.01" name="rental_income" value="${state.rental_income}"></label>
                <label><span data-i18n="view.s469.label.other_passive_income">Other passive income (K-1 LP, etc.) ($)</span>
                    <input type="number" step="0.01" name="other_passive_income" value="${state.other_passive_income}"></label>
                <label><span data-i18n="view.s469.label.suspended">Suspended PAL carryover ($)</span>
                    <input type="number" step="0.01" name="suspended_pal_carryover" value="${state.suspended_pal_carryover}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s469.label.hours_rentals">Your hours in real estate this year</span>
                    <input type="number" step="1" name="hours_in_rentals" value="${state.hours_in_rentals}"></label>
                <label><span data-i18n="view.s469.label.hours_other">Hours in other paid work</span>
                    <input type="number" step="1" name="hours_in_other_work" value="${state.hours_in_other_work}"></label>
                <label><span data-i18n="view.s469.label.is_spouse_rep">Spouse qualifies as REP?</span>
                    <input type="checkbox" name="is_spouse_rep" ${state.is_spouse_rep ? 'checked' : ''}></label>
                <label><span data-i18n="view.s469.label.spouse_hours">Spouse hours in rentals</span>
                    <input type="number" step="1" name="spouse_hours_in_rentals" value="${state.spouse_hours_in_rentals}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s469.label.mp_500">Material part: &gt;500 hrs in activity?</span>
                    <input type="checkbox" name="mp_more_than_500" ${state.mp_more_than_500 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s469.label.mp_only_one">You did substantially all the work?</span>
                    <input type="checkbox" name="mp_only_one_doing_most" ${state.mp_only_one_doing_most ? 'checked' : ''}></label>
                <label><span data-i18n="view.s469.label.mp_100">&gt;100 hrs + no one else more?</span>
                    <input type="checkbox" name="mp_100_and_no_one_more" ${state.mp_100_and_no_one_more ? 'checked' : ''}></label>
                <label><span data-i18n="view.s469.label.mp_fc">Regular, continuous, substantial (F&C)?</span>
                    <input type="checkbox" name="mp_facts_circumstances" ${state.mp_facts_circumstances ? 'checked' : ''}></label>
                <label><span data-i18n="view.s469.label.self_rental">Self-rental to your own biz?</span>
                    <input type="checkbox" name="self_rental_to_own_biz" ${state.self_rental_to_own_biz ? 'checked' : ''}></label>
                <label><span data-i18n="view.s469.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s469.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s469-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s469.h2.case_law">Leading REP case law</h2>
            <ul class="muted small">
                <li data-i18n="view.s469.case.hakkak">Hakkak v. Comm'r (2020) — DENIED: contemporaneous log lacking, "ballpark guesses"</li>
                <li data-i18n="view.s469.case.miller">Miller v. Comm'r (2010) — GRANTED: full-time builder, kept time log, 1000+ hrs</li>
                <li data-i18n="view.s469.case.bailey">Bailey v. Comm'r (2001) — DENIED: full-time IBM engineer, can't satisfy &gt; 50% test</li>
                <li data-i18n="view.s469.case.agarwal">Agarwal v. Comm'r (2009) — DENIED: physician spouse not REP, &gt;50% in medicine</li>
                <li data-i18n="view.s469.case.lapid">Lapid v. Comm'r (2017) — DENIED: failed material participation per activity test</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s469.h2.aggregation">§ 1.469-9(g) Aggregation election</h2>
            <p class="muted small" data-i18n="view.s469.aggregation.body">
                If you have multiple rentals, file an aggregation election treating ALL as ONE
                activity. Then 750-hr / 100-hr / material participation tests applied across
                aggregate, not per property. ELECTION IS IRREVOCABLE absent material change of
                facts. Attach statement to year-1 return. Without this, each property tested
                separately and material participation often fails.
            </p>
        </div>
    `;
    document.getElementById('s469-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.rental_loss = Number(fd.get('rental_loss')) || 0;
        state.rental_income = Number(fd.get('rental_income')) || 0;
        state.other_passive_income = Number(fd.get('other_passive_income')) || 0;
        state.suspended_pal_carryover = Number(fd.get('suspended_pal_carryover')) || 0;
        state.hours_in_rentals = Number(fd.get('hours_in_rentals')) || 0;
        state.hours_in_other_work = Number(fd.get('hours_in_other_work')) || 0;
        state.is_spouse_rep = !!fd.get('is_spouse_rep');
        state.spouse_hours_in_rentals = Number(fd.get('spouse_hours_in_rentals')) || 0;
        state.mp_more_than_500 = !!fd.get('mp_more_than_500');
        state.mp_only_one_doing_most = !!fd.get('mp_only_one_doing_most');
        state.mp_100_and_no_one_more = !!fd.get('mp_100_and_no_one_more');
        state.mp_facts_circumstances = !!fd.get('mp_facts_circumstances');
        state.self_rental_to_own_biz = !!fd.get('self_rental_to_own_biz');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s469-output');
    if (!el) return;
    const rep_750 = state.hours_in_rentals >= 750;
    const rep_50pct = state.hours_in_other_work === 0
        ? state.hours_in_rentals > 0
        : state.hours_in_rentals > state.hours_in_other_work;
    const isRep = (rep_750 && rep_50pct)
        || (state.is_spouse_rep && state.spouse_hours_in_rentals >= 750);
    const isMaterialParticipation = state.mp_more_than_500
        || state.mp_only_one_doing_most
        || state.mp_100_and_no_one_more
        || state.mp_facts_circumstances;
    const lossesUnlocked = isRep && isMaterialParticipation;
    const passiveAvailable = state.rental_income + state.other_passive_income;
    const totalLoss = state.rental_loss + state.suspended_pal_carryover;
    const usableThisYear = lossesUnlocked
        ? totalLoss
        : Math.min(totalLoss, passiveAvailable);
    const suspendedNextYear = lossesUnlocked ? 0 : Math.max(0, totalLoss - passiveAvailable);
    const taxSavings = usableThisYear * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s469.h2.result">Status + loss flow</h2>
            <div class="cards">
                <div class="card ${isRep ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s469.card.rep">REP qualified?</div>
                    <div class="value">${isRep ? esc(t('view.s469.status.yes')) : esc(t('view.s469.status.no'))}</div>
                </div>
                <div class="card ${rep_750 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s469.card.test_750">≥750 hrs test</div>
                    <div class="value">${rep_750 ? esc(t('view.s469.status.yes')) : esc(t('view.s469.status.no'))}</div>
                </div>
                <div class="card ${rep_50pct ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s469.card.test_50">&gt;50% services test</div>
                    <div class="value">${rep_50pct ? esc(t('view.s469.status.yes')) : esc(t('view.s469.status.no'))}</div>
                </div>
                <div class="card ${isMaterialParticipation ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s469.card.mp">Material participation</div>
                    <div class="value">${isMaterialParticipation ? esc(t('view.s469.status.yes')) : esc(t('view.s469.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s469.card.total_loss">Total losses (year + carry)</div>
                    <div class="value">$${totalLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s469.card.usable">Usable this year</div>
                    <div class="value">$${usableThisYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${suspendedNextYear > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s469.card.suspended_next">Suspended → next year</div>
                    <div class="value">$${suspendedNextYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s469.card.tax_savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.self_rental_to_own_biz ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s469.warning.self_rental">
                    Self-rental trap: under § 469 net rental INCOME from property leased to your own
                    active business is RECHARACTERIZED as non-passive — cannot absorb other passive
                    losses. Net rental LOSS stays passive. Don't let your CPA put the property
                    in the same entity.
                </p>
            ` : ''}
        </div>
    `;
}
