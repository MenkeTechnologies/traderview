// IRC § 1092 — Straddle Rules + Mixed-Straddle Election.
// A "straddle" = offsetting positions in actively traded personal property.
// Loss on one leg deferred to extent of unrealized gain on offsetting leg.
// Holding period TOLLED during straddle. Capitalized interest + carrying charges (§ 263(g)).
// § 1092(b)(2) MIXED-STRADDLE ELECTION: identify positions for § 1256 60/40 + non-§ 1256 treatment.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    loss_leg_realized: 0,
    gain_leg_unrealized: 0,
    has_mixed_straddle_election: false,
    is_qualified_section_1256: false,
    interest_capitalized: 0,
    short_term_capital_loss: 0,
    long_term_capital_loss: 0,
    marginal_rate: 0.32,
    ltcg_rate: 0.20,
};

export async function renderSection1092(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1092.h1.title">// § 1092 STRADDLE / MIXED-STRADDLE</span></h1>
        <p class="muted small" data-i18n="view.s1092.hint.intro">
            Straddle = offsetting positions in actively traded property. Loss on one leg
            <strong>DEFERRED</strong> to extent of unrealized gain on offsetting leg. Holding
            period <strong>TOLLED</strong> during straddle. <strong>§ 263(g):</strong> interest +
            carrying charges CAPITALIZED. <strong>§ 1092(b)(2) Mixed-Straddle Election:</strong>
            identify positions to allow § 1256 60/40 treatment + non-§ 1256 leg.
            <strong>§ 1233(d) Killer Straddle:</strong> short-against-the-box trap.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1092.h2.inputs">Inputs</h2>
            <form id="s1092-form" class="inline-form">
                <label><span data-i18n="view.s1092.label.loss_realized">Loss leg realized ($)</span>
                    <input type="number" step="100" name="loss_leg_realized" value="${state.loss_leg_realized}"></label>
                <label><span data-i18n="view.s1092.label.gain_unrealized">Gain leg unrealized ($)</span>
                    <input type="number" step="100" name="gain_leg_unrealized" value="${state.gain_leg_unrealized}"></label>
                <label><span data-i18n="view.s1092.label.mixed_election">Mixed-Straddle Election made?</span>
                    <input type="checkbox" name="has_mixed_straddle_election" ${state.has_mixed_straddle_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1092.label.section_1256">One leg § 1256 contract?</span>
                    <input type="checkbox" name="is_qualified_section_1256" ${state.is_qualified_section_1256 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1092.label.interest">Interest + carrying charges to capitalize ($)</span>
                    <input type="number" step="100" name="interest_capitalized" value="${state.interest_capitalized}"></label>
                <label><span data-i18n="view.s1092.label.st_loss">ST capital loss in straddle ($)</span>
                    <input type="number" step="100" name="short_term_capital_loss" value="${state.short_term_capital_loss}"></label>
                <label><span data-i18n="view.s1092.label.lt_loss">LT capital loss in straddle ($)</span>
                    <input type="number" step="100" name="long_term_capital_loss" value="${state.long_term_capital_loss}"></label>
                <label><span data-i18n="view.s1092.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s1092.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s1092.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1092-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1092.h2.identification">Straddle identification</h2>
            <ul class="muted small">
                <li data-i18n="view.s1092.id.delta">Offsetting positions with substantial reduction in risk of loss</li>
                <li data-i18n="view.s1092.id.actively_traded">Actively traded personal property (stocks, options, futures, currencies)</li>
                <li data-i18n="view.s1092.id.presumption">Presumption: same-asset hedges (e.g., long stock + protective put)</li>
                <li data-i18n="view.s1092.id.qualified_covered">"Qualified covered calls" (§ 1092(c)(4)) EXEMPT — OTM call against long stock</li>
                <li data-i18n="view.s1092.id.hedging">§ 1221(a)(7) hedging transactions: ordinary character, exempt from § 1092</li>
                <li data-i18n="view.s1092.id.identified_straddle">"Identified straddle" (§ 1092(a)(2)): elect within 30 days for special treatment</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1092.h2.mixed_election">§ 1092(b)(2) Mixed-Straddle Election</h2>
            <p class="muted small" data-i18n="view.s1092.mixed.body">
                When one position is § 1256 (futures, broad-based index options) and another is not,
                taxpayer may elect to: (1) treat § 1256 position as non-§ 1256 (no 60/40 + no MTM), OR
                (2) net the positions identifying each pre-straddle. <strong>Annual election</strong>;
                identification within first day of straddle creation. Useful for tax-rate arbitrage
                between 60/40 (LT 23.8% blended max) vs full ordinary treatment.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1092.h2.related">Related anti-abuse rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s1092.rel.1259">§ 1259 constructive sale (delta-1 or substantially eliminated)</li>
                <li data-i18n="view.s1092.rel.1091">§ 1091 wash sale + § 1092 stack: complex interaction</li>
                <li data-i18n="view.s1092.rel.1233">§ 1233 short sale holding period</li>
                <li data-i18n="view.s1092.rel.1234">§ 1234A capital loss on terminated rights</li>
                <li data-i18n="view.s1092.rel.246">§ 246(c) holding period requirement for QDI ÷ DRD</li>
                <li data-i18n="view.s1092.rel.475f">§ 475(f) trader MTM overrides § 1092 (all ordinary anyway)</li>
            </ul>
        </div>
    `;
    document.getElementById('s1092-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.loss_leg_realized = Number(fd.get('loss_leg_realized')) || 0;
        state.gain_leg_unrealized = Number(fd.get('gain_leg_unrealized')) || 0;
        state.has_mixed_straddle_election = !!fd.get('has_mixed_straddle_election');
        state.is_qualified_section_1256 = !!fd.get('is_qualified_section_1256');
        state.interest_capitalized = Number(fd.get('interest_capitalized')) || 0;
        state.short_term_capital_loss = Number(fd.get('short_term_capital_loss')) || 0;
        state.long_term_capital_loss = Number(fd.get('long_term_capital_loss')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1092-output');
    if (!el) return;
    const deferredLoss = Math.min(state.loss_leg_realized, state.gain_leg_unrealized);
    const allowedLoss = state.loss_leg_realized - deferredLoss;
    const taxSavedNow = allowedLoss * state.marginal_rate;
    const taxIfMixed = state.is_qualified_section_1256 && state.has_mixed_straddle_election
        ? state.loss_leg_realized * (0.60 * state.ltcg_rate + 0.40 * state.marginal_rate)
        : state.loss_leg_realized * state.marginal_rate;
    const taxDifferenceFromMixed = state.loss_leg_realized * state.marginal_rate - taxIfMixed;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1092.h2.result">§ 1092 outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1092.card.realized">Realized loss</div>
                    <div class="value">$${state.loss_leg_realized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1092.card.deferred">Loss deferred (unrealized gain offset)</div>
                    <div class="value">$${deferredLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1092.card.allowed">Allowed now</div>
                    <div class="value">$${allowedLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1092.card.tax_saved">Year-1 tax saved</div>
                    <div class="value">$${taxSavedNow.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1092.card.interest">Interest capitalized § 263(g)</div>
                    <div class="value">$${state.interest_capitalized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.is_qualified_section_1256 ? `
                    <div class="card ${state.has_mixed_straddle_election ? 'pos' : 'neg'}">
                        <div class="label" data-i18n="view.s1092.card.mixed_election">Mixed election impact</div>
                        <div class="value">$${taxDifferenceFromMixed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
