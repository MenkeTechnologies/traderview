// IRC § 1402 — Self-Employment Tax base.
// 15.3% on net SE earnings × 0.9235 (employer-portion deduction): 12.4% Social Security + 2.9% Medicare.
// Cap: $168,600 SS wage base (2024); Medicare uncapped.
// Additional 0.9% Medicare > $200k single / $250k MFJ.
// Half-SE-tax deduction § 164(f) above-the-line on Schedule 1.

import { currentViewToken, viewIsCurrent } from '../app.js';

const SS_BASE_2024 = 168_600;
const SS_RATE = 0.124;
const MEDICARE_RATE = 0.029;
const ADD_MEDICARE_RATE = 0.009;
const ADD_MEDICARE_THRESH_SINGLE = 200_000;
const ADD_MEDICARE_THRESH_MFJ = 250_000;
const NET_SE_ADJUST = 0.9235;

let state = {
    schedule_c_net: 0,
    schedule_k1_se: 0,
    w2_wages_already_paid: 0,
    is_mfj: false,
    spouse_se_earnings: 0,
    income_marginal: 0.32,
};

export async function renderSection1402(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1402.h1.title">// § 1402 SELF-EMPLOYMENT TAX</span></h1>
        <p class="muted small" data-i18n="view.s1402.hint.intro">
            <strong>15.3%</strong> on net SE × <strong>0.9235</strong>: 12.4% SS + 2.9% Medicare.
            <strong>SS wage base 2024: $168,600</strong>; Medicare uncapped. <strong>Additional
            0.9% Medicare</strong> over $200k single / $250k MFJ. <strong>Half-SE-tax deduction</strong>
            § 164(f) above-the-line. W-2 wages from same employer REDUCE SS-base SE tax dollar-for-dollar.
            Form SE attached to 1040.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1402.h2.inputs">Inputs</h2>
            <form id="s1402-form" class="inline-form">
                <label><span data-i18n="view.s1402.label.sched_c">Schedule C net profit ($)</span>
                    <input type="number" step="1000" name="schedule_c_net" value="${state.schedule_c_net}"></label>
                <label><span data-i18n="view.s1402.label.k1">Schedule K-1 SE earnings ($)</span>
                    <input type="number" step="1000" name="schedule_k1_se" value="${state.schedule_k1_se}"></label>
                <label><span data-i18n="view.s1402.label.w2">W-2 wages already paid (SS-capped portion) ($)</span>
                    <input type="number" step="1000" name="w2_wages_already_paid" value="${state.w2_wages_already_paid}"></label>
                <label><span data-i18n="view.s1402.label.mfj">MFJ?</span>
                    <input type="checkbox" name="is_mfj" ${state.is_mfj ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1402.label.spouse">Spouse SE earnings ($)</span>
                    <input type="number" step="1000" name="spouse_se_earnings" value="${state.spouse_se_earnings}"></label>
                <label><span data-i18n="view.s1402.label.marginal">Income marginal %</span>
                    <input type="number" step="0.01" name="income_marginal" value="${state.income_marginal}"></label>
                <button class="primary" type="submit" data-i18n="view.s1402.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1402-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1402.h2.exclusions">Income NOT subject to SE tax</h2>
            <ul class="muted small">
                <li data-i18n="view.s1402.excl.investment">Investment income: dividends, interest, capital gains (incl. trader gains)</li>
                <li data-i18n="view.s1402.excl.rental">Rental real estate income (passive)</li>
                <li data-i18n="view.s1402.excl.s_corp_dist">S-corp distributions to shareholder-employees (above reasonable comp)</li>
                <li data-i18n="view.s1402.excl.partner_lp">Limited partner distributive share (excl. § 1402(a)(13))</li>
                <li data-i18n="view.s1402.excl.tts_trader">§ 475(f) trader: TTS trading income NOT SE (cap gains, not personal services)</li>
                <li data-i18n="view.s1402.excl.lump_sum">Lump-sum benefits from earlier employment</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1402.h2.optimizations">SE tax optimization</h2>
            <ul class="muted small">
                <li data-i18n="view.s1402.opt.s_corp">S-corp election + reasonable comp: distributions escape SE tax</li>
                <li data-i18n="view.s1402.opt.lp_status">LLC member-managed: § 1402(a)(13) protection (LP-style)</li>
                <li data-i18n="view.s1402.opt.qbi">QBI deduction § 199A still applies to SE income</li>
                <li data-i18n="view.s1402.opt.half_se">Half-SE-tax deduction § 164(f) flows to Schedule 1</li>
                <li data-i18n="view.s1402.opt.health_ins">Self-employed health insurance § 162(l) above-the-line</li>
                <li data-i18n="view.s1402.opt.retirement">Solo 401(k) / SEP / SIMPLE max with QBI-friendly structure</li>
                <li data-i18n="view.s1402.opt.spouse_split">Spouse partnership splitting: each gets own SS-base wage cap</li>
            </ul>
        </div>
    `;
    document.getElementById('s1402-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.schedule_c_net = Number(fd.get('schedule_c_net')) || 0;
        state.schedule_k1_se = Number(fd.get('schedule_k1_se')) || 0;
        state.w2_wages_already_paid = Number(fd.get('w2_wages_already_paid')) || 0;
        state.is_mfj = !!fd.get('is_mfj');
        state.spouse_se_earnings = Number(fd.get('spouse_se_earnings')) || 0;
        state.income_marginal = Number(fd.get('income_marginal')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1402-output');
    if (!el) return;
    const grossSe = state.schedule_c_net + state.schedule_k1_se;
    const netSeEarnings = grossSe * NET_SE_ADJUST;
    const ssRoom = Math.max(0, SS_BASE_2024 - state.w2_wages_already_paid);
    const ssSubject = Math.min(netSeEarnings, ssRoom);
    const ssTax = ssSubject * SS_RATE;
    const medicareTax = netSeEarnings * MEDICARE_RATE;
    const addMedicareThresh = state.is_mfj ? ADD_MEDICARE_THRESH_MFJ : ADD_MEDICARE_THRESH_SINGLE;
    const totalEarnedIncome = grossSe + state.w2_wages_already_paid + (state.is_mfj ? state.spouse_se_earnings : 0);
    const addMedicareSubject = Math.max(0, totalEarnedIncome - addMedicareThresh);
    const addMedicareTax = addMedicareSubject * ADD_MEDICARE_RATE;
    const totalSe = ssTax + medicareTax + addMedicareTax;
    const halfSeDeduction = (ssTax + medicareTax) / 2;
    const incomeTaxSaved = halfSeDeduction * state.income_marginal;
    const netCost = totalSe - incomeTaxSaved;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1402.h2.result">SE tax calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1402.card.gross_se">Gross SE earnings</div>
                    <div class="value">$${grossSe.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1402.card.net_se">Net SE (× 0.9235)</div>
                    <div class="value">$${netSeEarnings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1402.card.ss_room">SS-base room remaining</div>
                    <div class="value">$${ssRoom.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1402.card.ss_tax">Social Security tax (12.4%)</div>
                    <div class="value">$${ssTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1402.card.medicare">Medicare (2.9%)</div>
                    <div class="value">$${medicareTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${addMedicareTax > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s1402.card.add_medicare">Additional 0.9% Medicare</div>
                        <div class="value">$${addMedicareTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card neg">
                    <div class="label" data-i18n="view.s1402.card.total_se">Total SE tax</div>
                    <div class="value">$${totalSe.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1402.card.half_ded">Half-SE-tax deduction</div>
                    <div class="value">$${halfSeDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1402.card.income_saved">Income tax saved</div>
                    <div class="value">$${incomeTaxSaved.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1402.card.net_cost">Net SE cost</div>
                    <div class="value">$${netCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
