// IRC § 1295 — QEF (Qualified Electing Fund) Election.
// Annual current inclusion of pro-rata ORDINARY EARNINGS + NET CAPITAL GAIN.
// Avoids § 1291 interest charge — preserves character (ordinary vs capital).
// Requires PFIC Annual Information Statement (PFIC AIS) from foreign corp.
// Election made on Form 8621 attached to timely return; first-year of ownership.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    qef_ordinary_earnings: 0,
    qef_net_capital_gain: 0,
    actual_distribution: 0,
    prior_qef_inclusions: 0,
    has_ais: false,
    first_year_election: true,
    purging_election: false,
    holding_period_years: 0,
    shareholder_status: 'individual',
};

export async function renderSection1295(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1295.h1.title">// § 1295 PFIC QEF</span></h1>
        <p class="muted small" data-i18n="view.s1295.hint.intro">
            <strong>QEF</strong> = annual current inclusion of pro-rata <strong>ORDINARY EARNINGS</strong> +
            <strong>NET CAPITAL GAIN</strong> of PFIC. <strong>Avoids § 1291 interest charge</strong> AND preserves
            CHARACTER (ordinary vs LTCG). <strong>Requires PFIC AIS</strong> (Annual Information Statement)
            from foreign corp. Election: Form 8621 with timely-filed return; <strong>first-year</strong> of
            shareholder ownership best. Late = combine with PURGING election.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1295.h2.inputs">Inputs</h2>
            <form id="s1295-form" class="inline-form">
                <label><span data-i18n="view.s1295.label.ord">QEF ordinary earnings inclusion ($)</span>
                    <input type="number" step="0.01" name="qef_ordinary_earnings" value="${state.qef_ordinary_earnings}"></label>
                <label><span data-i18n="view.s1295.label.ncg">QEF net capital gain inclusion ($)</span>
                    <input type="number" step="0.01" name="qef_net_capital_gain" value="${state.qef_net_capital_gain}"></label>
                <label><span data-i18n="view.s1295.label.actual">Actual distribution received ($)</span>
                    <input type="number" step="0.01" name="actual_distribution" value="${state.actual_distribution}"></label>
                <label><span data-i18n="view.s1295.label.prior">Prior years cumulative QEF inclusions ($)</span>
                    <input type="number" step="0.01" name="prior_qef_inclusions" value="${state.prior_qef_inclusions}"></label>
                <label><span data-i18n="view.s1295.label.ais">Has PFIC AIS?</span>
                    <input type="checkbox" name="has_ais" ${state.has_ais ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1295.label.first">First-year election?</span>
                    <input type="checkbox" name="first_year_election" ${state.first_year_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1295.label.purge">Purging election made?</span>
                    <input type="checkbox" name="purging_election" ${state.purging_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1295.label.holding">Holding period years</span>
                    <input type="number" step="1" name="holding_period_years" value="${state.holding_period_years}"></label>
                <label><span data-i18n="view.s1295.label.status">Shareholder status</span>
                    <select name="shareholder_status">
                        <option value="individual" ${state.shareholder_status === 'individual' ? 'selected' : ''}>Individual</option>
                        <option value="corporation" ${state.shareholder_status === 'corporation' ? 'selected' : ''}>C-Corporation</option>
                        <option value="partnership" ${state.shareholder_status === 'partnership' ? 'selected' : ''}>Partnership / S-corp (pass-through)</option>
                        <option value="trust" ${state.shareholder_status === 'trust' ? 'selected' : ''}>Trust / Estate</option>
                    </select>
                </label>
                <button class="primary" type="submit" data-i18n="view.s1295.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1295-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1295.h2.mechanics">QEF mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.s1295.mech.eligibility">Must elect for first year shareholder treats foreign corp as PFIC</li>
                <li data-i18n="view.s1295.mech.late">Late election available only with PURGING (deemed sale at § 1291 rates)</li>
                <li data-i18n="view.s1295.mech.character">Inclusion preserves character — ordinary earnings + LTCG separate inclusion lines</li>
                <li data-i18n="view.s1295.mech.basis">Basis += inclusions; basis -= subsequent distributions of PTI</li>
                <li data-i18n="view.s1295.mech.ptep">Previously taxed income (PTI) regime prevents double-taxation on later distributions</li>
                <li data-i18n="view.s1295.mech.timing">Inclusion in shareholder's year that ends with or within PFIC's taxable year</li>
                <li data-i18n="view.s1295.mech.deferral">Optional § 1294 deferral election: defer with interest charge until distribution</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1295.h2.ais">PFIC Annual Information Statement (AIS)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1295.ais.required">Required from foreign corp for QEF election to be valid</li>
                <li data-i18n="view.s1295.ais.contents">Contents: ordinary earnings + net capital gain attributable to shareholder + distributions + adjustments</li>
                <li data-i18n="view.s1295.ais.consent">Must include consent to U.S. examination of corporate records</li>
                <li data-i18n="view.s1295.ais.protective">Protective Statement available if AIS not yet received — preserve QEF without AIS</li>
                <li data-i18n="view.s1295.ais.controlled">Controlled QEF: shareholder + family + related own > 50% — can compel AIS</li>
                <li data-i18n="view.s1295.ais.alternative">Alternative: shareholder may obtain via legal action if rights established</li>
                <li data-i18n="view.s1295.ais.no_ais">No AIS = no QEF eligibility = default § 1291 regime applies</li>
            </ul>
        </div>
    `;
    document.getElementById('s1295-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.qef_ordinary_earnings = Number(fd.get('qef_ordinary_earnings')) || 0;
        state.qef_net_capital_gain = Number(fd.get('qef_net_capital_gain')) || 0;
        state.actual_distribution = Number(fd.get('actual_distribution')) || 0;
        state.prior_qef_inclusions = Number(fd.get('prior_qef_inclusions')) || 0;
        state.has_ais = !!fd.get('has_ais');
        state.first_year_election = !!fd.get('first_year_election');
        state.purging_election = !!fd.get('purging_election');
        state.holding_period_years = Number(fd.get('holding_period_years')) || 0;
        state.shareholder_status = fd.get('shareholder_status');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1295-output');
    if (!el) return;
    const eligible = state.has_ais && (state.first_year_election || state.purging_election);
    const totalInclusion = state.qef_ordinary_earnings + state.qef_net_capital_gain;
    const isIndividual = state.shareholder_status === 'individual';
    const ordinaryRate = isIndividual ? 0.37 : 0.21;
    const capitalRate = isIndividual ? 0.20 : 0.21;
    const taxOrdinary = state.qef_ordinary_earnings * ordinaryRate;
    const taxCapital = state.qef_net_capital_gain * capitalRate;
    const totalTax = taxOrdinary + taxCapital;
    const distributionOfPTI = Math.min(state.actual_distribution, state.prior_qef_inclusions + totalInclusion);
    const distributionTaxable = Math.max(0, state.actual_distribution - distributionOfPTI);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1295.h2.result">QEF computation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1295.card.eligible">Eligible for QEF?</div>
                    <div class="value">${eligible ? esc(t('view.s1295.status.yes')) : esc(t('view.s1295.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1295.card.ordinary">Ordinary earnings inclusion</div>
                    <div class="value">$${state.qef_ordinary_earnings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1295.card.capital">Net capital gain inclusion</div>
                    <div class="value">$${state.qef_net_capital_gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1295.card.total_inclusion">Total inclusion</div>
                    <div class="value">$${totalInclusion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1295.card.tax_ord">Tax on ordinary (${(ordinaryRate * 100).toFixed(0)}%)</div>
                    <div class="value">$${taxOrdinary.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1295.card.tax_cap">Tax on capital gain (${(capitalRate * 100).toFixed(0)}%)</div>
                    <div class="value">$${taxCapital.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1295.card.pti">PTI distribution (tax-free)</div>
                    <div class="value">$${distributionOfPTI.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1295.card.total">Total QEF tax</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!eligible ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1295.not_eligible_note">
                    Not eligible for QEF without PFIC AIS + first-year (or purging) election. Default
                    § 1291 regime applies: highest ordinary rate + interest charge back to start.
                </p>
            ` : ''}
        </div>
    `;
}
