// § 475(f) Mark-to-Market election tracker.
// Permanent ordinary treatment + no wash sales + Schedule C losses unlimited.
// Election deadline: April 15 of YEAR (NOT prior year). New entities: 75 days from formation.
// Late election: Rev. Proc. 99-17 OR § 9100 relief (Vines v. Comm'r).
// Form 3115 required to revoke. Mark all positions to market on Dec 31.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    election_year: new Date().getFullYear() + 1,
    has_tts: false,
    avg_annual_pnl: 0,
    avg_annual_wash_loss: 0,
    capital_loss_carryover: 0,
    marginal_rate: 0.32,
};

export async function renderSection475f(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.475f.h1.title">// § 475(f) MTM ELECTION</span></h1>
        <p class="muted small" data-i18n="view.475f.hint.intro">
            Trader Tax Status holders elect ordinary treatment: <strong>all positions marked
            to market Dec 31</strong>, gains/losses ordinary, NO wash-sale rules, NO $3,000
            annual capital loss limit. Election deadline: <strong>April 15</strong> of the year
            you want it effective (for new entities: 75 days from formation). Revocation requires
            Form 3115.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.475f.h2.timing">Election timing</h2>
            <ul class="muted small">
                <li data-i18n="view.475f.timing.individual">Individual: by April 15 of election year (e.g., for 2026 effective, file by April 15, 2026)</li>
                <li data-i18n="view.475f.timing.entity">New entity (LLC / S-corp): within 75 days of formation</li>
                <li data-i18n="view.475f.timing.statement">Attach election statement to extension OR original return</li>
                <li data-i18n="view.475f.timing.form_3115_in">Form 3115 (Method Change) due with first MTM return</li>
                <li data-i18n="view.475f.timing.late">Late? Vines § 9100 relief (good cause) OR Rev. Proc. 99-17</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.475f.h2.inputs">Inputs</h2>
            <form id="m475-form" class="inline-form">
                <label><span data-i18n="view.475f.label.election_year">Election effective year</span>
                    <input type="number" step="1" name="election_year" value="${state.election_year}"></label>
                <label><span data-i18n="view.475f.label.has_tts">TTS qualified?</span>
                    <input type="checkbox" name="has_tts" ${state.has_tts ? 'checked' : ''}></label>
                <label><span data-i18n="view.475f.label.avg_pnl">Avg annual P&L (last 3 yr) ($)</span>
                    <input type="number" step="1000" name="avg_annual_pnl" value="${state.avg_annual_pnl}"></label>
                <label><span data-i18n="view.475f.label.avg_wash">Avg annual wash-sale loss disallowed ($)</span>
                    <input type="number" step="100" name="avg_annual_wash_loss" value="${state.avg_annual_wash_loss}"></label>
                <label><span data-i18n="view.475f.label.cap_carryover">Existing capital loss carryover ($)</span>
                    <input type="number" step="1000" name="capital_loss_carryover" value="${state.capital_loss_carryover}"></label>
                <label><span data-i18n="view.475f.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.475f.btn.evaluate">Evaluate</button>
            </form>
        </div>
        <div id="m475-output"></div>
    `;
    document.getElementById('m475-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.election_year = Number(fd.get('election_year')) || new Date().getFullYear() + 1;
        state.has_tts = !!fd.get('has_tts');
        state.avg_annual_pnl = Number(fd.get('avg_annual_pnl')) || 0;
        state.avg_annual_wash_loss = Number(fd.get('avg_annual_wash_loss')) || 0;
        state.capital_loss_carryover = Number(fd.get('capital_loss_carryover')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('m475-output');
    if (!el) return;
    const electionDeadline = `April 15, ${state.election_year}`;
    const annualWashSaving = state.avg_annual_wash_loss * state.marginal_rate;
    const annualLossUnlock = state.avg_annual_pnl < 0
        ? Math.min(Math.abs(state.avg_annual_pnl) - 3000, Math.abs(state.avg_annual_pnl)) * state.marginal_rate
        : 0;
    const carryoverStranded = state.capital_loss_carryover;
    const recommendation = state.has_tts
        ? (state.avg_annual_pnl < 0 ? t('view.475f.rec.elect_loss') : (state.avg_annual_wash_loss > 5000 ? t('view.475f.rec.elect_wash') : t('view.475f.rec.consider')))
        : t('view.475f.rec.no_tts');
    const recCls = state.has_tts && (state.avg_annual_pnl < 0 || state.avg_annual_wash_loss > 5000) ? 'pos' : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.475f.h2.result">Evaluation</h2>
            <div class="cards">
                <div class="card ${state.has_tts ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.475f.card.deadline">Election deadline</div>
                    <div class="value">${esc(electionDeadline)}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.475f.card.wash_saving">Annual wash-sale tax saving</div>
                    <div class="value">$${annualWashSaving.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.475f.card.loss_unlock">Annual loss unlock (vs $3k cap)</div>
                    <div class="value">$${annualLossUnlock.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${carryoverStranded > 50_000 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.475f.card.stranded">Stranded capital carryover</div>
                    <div class="value">$${carryoverStranded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${recCls}">
                    <div class="label" data-i18n="view.475f.card.recommendation">Recommendation</div>
                    <div class="value">${esc(recommendation)}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.475f.h2.tradeoffs">Tradeoffs</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.475f.th.aspect">Aspect</th>
                    <th data-i18n="view.475f.th.before">Before MTM</th>
                    <th data-i18n="view.475f.th.after">After § 475(f)</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.475f.row.character">Character</td><td>Capital gains (LT preferred 15/20%)</td><td>Ordinary (your marginal rate)</td></tr>
                    <tr><td data-i18n="view.475f.row.wash_sales">Wash sale rules</td><td>APPLY</td><td>EXEMPT</td></tr>
                    <tr><td data-i18n="view.475f.row.loss_limit">Loss limit</td><td>$3,000/yr against ordinary</td><td>UNLIMITED ordinary</td></tr>
                    <tr><td data-i18n="view.475f.row.long_term">LTCG benefit</td><td>YES (15/20% if &gt; 1 yr)</td><td>FORFEITED</td></tr>
                    <tr><td data-i18n="view.475f.row.qbi">§ 199A QBI</td><td>NO</td><td>POTENTIALLY (if SSTB analysis OK)</td></tr>
                    <tr><td data-i18n="view.475f.row.se_tax">SE tax</td><td>NO</td><td>NO (TIT not SE)</td></tr>
                    <tr><td data-i18n="view.475f.row.year_end">Year-end</td><td>Realized only</td><td>Mark all to Dec 31 close</td></tr>
                    <tr><td data-i18n="view.475f.row.revoke">Revocation</td><td>—</td><td>Form 3115, may take years</td></tr>
                </tbody>
            </table>
        </div>
    `;
}
