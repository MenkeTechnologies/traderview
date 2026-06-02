// IRC § 172 — Net Operating Loss (NOL) Carryforward.
// TCJA 2017: pre-2018 NOLs 2-yr carryback + 20-yr forward (100% offset).
// Post-2017 NOLs: NO carryback (except CARES 2018-2020 5-yr), INDEFINITE forward,
// LIMITED TO 80% of taxable income. Excess business loss limit § 461(l) layered on top.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const POST_2017_OFFSET_LIMIT = 0.80;
const EBL_2024_SINGLE = 305_000;
const EBL_2024_MFJ = 610_000;

let state = {
    current_year_loss: 0,
    pre_2018_nol_remaining: 0,
    post_2017_nol_remaining: 0,
    current_year_taxable_income_before_nol: 0,
    is_mfj: false,
    is_business_loss: true,
    other_income_for_ebl: 0,
    marginal_rate: 0.32,
};

export async function renderSection172(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s172.h1.title">// § 172 NET OPERATING LOSS CARRYFORWARD</span></h1>
        <p class="muted small" data-i18n="view.s172.hint.intro">
            <strong>Pre-2018 NOLs:</strong> 2-yr carryback + 20-yr forward, 100% offset.
            <strong>Post-2017 NOLs:</strong> NO carryback (CARES 5-yr was 2018-2020 special),
            <strong>INDEFINITE forward</strong>, limited to <strong>80% of taxable income</strong>.
            <strong>§ 461(l) Excess Business Loss limit</strong> layered on top — $305k single /
            $610k MFJ (2024). Disallowed amount becomes NOL.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s172.h2.inputs">Inputs</h2>
            <form id="s172-form" class="inline-form">
                <label><span data-i18n="view.s172.label.current_loss">Current year operating loss ($)</span>
                    <input type="number" step="1000" name="current_year_loss" value="${state.current_year_loss}"></label>
                <label><span data-i18n="view.s172.label.pre_2018">Pre-2018 NOL remaining ($)</span>
                    <input type="number" step="1000" name="pre_2018_nol_remaining" value="${state.pre_2018_nol_remaining}"></label>
                <label><span data-i18n="view.s172.label.post_2017">Post-2017 NOL remaining ($)</span>
                    <input type="number" step="1000" name="post_2017_nol_remaining" value="${state.post_2017_nol_remaining}"></label>
                <label><span data-i18n="view.s172.label.current_ti">Current year TI BEFORE NOL ($)</span>
                    <input type="number" step="1000" name="current_year_taxable_income_before_nol" value="${state.current_year_taxable_income_before_nol}"></label>
                <label><span data-i18n="view.s172.label.mfj">MFJ?</span>
                    <input type="checkbox" name="is_mfj" ${state.is_mfj ? 'checked' : ''}></label>
                <label><span data-i18n="view.s172.label.business_loss">Business loss (subject to § 461(l))?</span>
                    <input type="checkbox" name="is_business_loss" ${state.is_business_loss ? 'checked' : ''}></label>
                <label><span data-i18n="view.s172.label.other_ebl">Other non-business income (for EBL) ($)</span>
                    <input type="number" step="1000" name="other_income_for_ebl" value="${state.other_income_for_ebl}"></label>
                <label><span data-i18n="view.s172.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s172.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s172-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s172.h2.cares_act">CARES Act + IRA + history</h2>
            <ul class="muted small">
                <li data-i18n="view.s172.cares.tcja">TCJA 2017: post-2017 NOLs no carryback, indefinite forward, 80% cap</li>
                <li data-i18n="view.s172.cares.2018_2020">CARES Act 2020: TY 2018-2020 NOLs → 5-yr carryback + temporary 100% offset</li>
                <li data-i18n="view.s172.cares.post_2020">2021+: Back to TCJA rules (no carryback, 80% limit)</li>
                <li data-i18n="view.s172.cares.farm_5yr">Farming NOLs: still 2-yr carryback option preserved</li>
                <li data-i18n="view.s172.cares.casualty">Casualty / property NOLs: special rules § 165(i)</li>
                <li data-i18n="view.s172.cares.382">§ 382: ownership change ≥ 50% in 3-yr period limits NOL usage</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s172.h2.ebl">§ 461(l) Excess Business Loss</h2>
            <p class="muted small" data-i18n="view.s172.ebl.body">
                Effective 2018 (originally TCJA, was suspended 2018-2020 then reactivated). Non-corp
                taxpayers: aggregate business loss exceeding <strong>$305k single / $610k MFJ
                (2024)</strong> is DISALLOWED. Excess becomes NOL carryforward. <strong>Inflation
                Reduction Act 2022 extended § 461(l)</strong> through 2028.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s172.h2.planning">Planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s172.plan.roth_recharacterize">Use NOL year to do large Roth conversion (absorbed at 0%)</li>
                <li data-i18n="view.s172.plan.harvest">Realize gains in NOL year to use up loss carryforward</li>
                <li data-i18n="view.s172.plan.s_corp_basis">S-corp basis must be tracked separately — limits flow-through loss usage</li>
                <li data-i18n="view.s172.plan.at_risk">§ 465 at-risk + § 469 passive activity layered on top of EBL + NOL</li>
                <li data-i18n="view.s172.plan.election">Pre-2018 carryback waiver election irrevocable — choose wisely</li>
                <li data-i18n="view.s172.plan.amt_nol">AMT NOL has separate tracking — not unified with regular NOL</li>
            </ul>
        </div>
    `;
    document.getElementById('s172-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.current_year_loss = Number(fd.get('current_year_loss')) || 0;
        state.pre_2018_nol_remaining = Number(fd.get('pre_2018_nol_remaining')) || 0;
        state.post_2017_nol_remaining = Number(fd.get('post_2017_nol_remaining')) || 0;
        state.current_year_taxable_income_before_nol = Number(fd.get('current_year_taxable_income_before_nol')) || 0;
        state.is_mfj = !!fd.get('is_mfj');
        state.is_business_loss = !!fd.get('is_business_loss');
        state.other_income_for_ebl = Number(fd.get('other_income_for_ebl')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s172-output');
    if (!el) return;
    // § 461(l) EBL
    const eblLimit = state.is_mfj ? EBL_2024_MFJ : EBL_2024_SINGLE;
    const grossLoss = state.current_year_loss;
    const allowedBusinessLoss = state.is_business_loss
        ? Math.min(grossLoss, state.other_income_for_ebl + eblLimit)
        : grossLoss;
    const disallowedToNol = grossLoss - allowedBusinessLoss;
    const newNolGenerated = Math.max(0, disallowedToNol);
    // NOL usage in current year (only if positive TI)
    const ti = state.current_year_taxable_income_before_nol;
    const pre_2018_used = Math.min(state.pre_2018_nol_remaining, Math.max(0, ti));
    const remainingTi = Math.max(0, ti - pre_2018_used);
    const post_2017_cap = remainingTi * POST_2017_OFFSET_LIMIT;
    const post_2017_used = Math.min(state.post_2017_nol_remaining, post_2017_cap);
    const finalTi = Math.max(0, remainingTi - post_2017_used);
    const remainingPre = state.pre_2018_nol_remaining - pre_2018_used;
    const remainingPost = state.post_2017_nol_remaining - post_2017_used + newNolGenerated;
    const taxSavings = (pre_2018_used + post_2017_used) * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s172.h2.result">Current year NOL flow</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s172.card.ebl_limit">§ 461(l) limit</div>
                    <div class="value">$${eblLimit.toLocaleString()}</div>
                </div>
                ${disallowedToNol > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s172.card.disallowed">EBL disallowed → NOL</div>
                        <div class="value">$${disallowedToNol.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card pos">
                    <div class="label" data-i18n="view.s172.card.pre_used">Pre-2018 NOL used</div>
                    <div class="value">$${pre_2018_used.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s172.card.post_used">Post-2017 NOL used (80% cap)</div>
                    <div class="value">$${post_2017_used.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s172.card.final_ti">Final TI after NOL</div>
                    <div class="value">$${finalTi.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s172.card.tax_saving">Tax savings from NOL</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s172.card.remaining_pre">Remaining pre-2018 NOL</div>
                    <div class="value">$${remainingPre.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s172.card.remaining_post">Remaining post-2017 NOL</div>
                    <div class="value">$${remainingPost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
