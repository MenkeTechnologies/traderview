// IRC § 1366 — S-Corp Pass-Through to Shareholder.
// Shareholder includes pro-rata share of S-corp income, loss, deduction, credit (separately stated).
// Items keep their CHARACTER (LTCG stays LTCG, qualified div stays qualified, § 1231, etc.).
// Pass-through tracked daily — per-day allocation if mid-year change.
// Loss limited to OUTSIDE BASIS (§ 1366(d)) + AT-RISK (§ 465) + PAL (§ 469).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    ownership_pct: 0,
    ordinary_business_income: 0,
    sep_stated_ltcg: 0,
    sep_stated_qualified_div: 0,
    sep_stated_s1231_gain: 0,
    sep_stated_charitable: 0,
    sep_stated_int_income: 0,
    days_owned: 365,
    outside_basis: 0,
    at_risk_amount: 0,
    is_passive: false,
    qbi_eligible_amount: 0,
    shareholder_marginal: 37,
};

export async function renderSection1366(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1366.h1.title">// § 1366 S-CORP PASS-THROUGH</span></h1>
        <p class="muted small" data-i18n="view.s1366.hint.intro">
            Shareholder includes pro-rata share of S-corp income, loss, deduction, credit. <strong>Items
            keep CHARACTER</strong> — LTCG stays LTCG, qualified div stays qualified, § 1231 separately.
            Pass-through tracked <strong>daily</strong> — per-day allocation for mid-year ownership change
            or election. <strong>Loss limited to OUTSIDE BASIS</strong> (§ 1366(d)), then <strong>AT-RISK</strong>
            (§ 465), then <strong>PAL</strong> (§ 469). Carryforward indefinitely until basis restored.
            <strong>QBI § 199A 20%</strong> deduction available.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1366.h2.inputs">Inputs</h2>
            <form id="s1366-form" class="inline-form">
                <label><span data-i18n="view.s1366.label.ownership">Ownership %</span>
                    <input type="number" step="0.01" name="ownership_pct" value="${state.ownership_pct}"></label>
                <label><span data-i18n="view.s1366.label.obi">Ordinary biz income / loss ($)</span>
                    <input type="number" step="10000" name="ordinary_business_income" value="${state.ordinary_business_income}"></label>
                <label><span data-i18n="view.s1366.label.ltcg">Sep stated LTCG ($)</span>
                    <input type="number" step="1000" name="sep_stated_ltcg" value="${state.sep_stated_ltcg}"></label>
                <label><span data-i18n="view.s1366.label.qdiv">Qualified dividends ($)</span>
                    <input type="number" step="1000" name="sep_stated_qualified_div" value="${state.sep_stated_qualified_div}"></label>
                <label><span data-i18n="view.s1366.label.s1231">§ 1231 gain / loss ($)</span>
                    <input type="number" step="1000" name="sep_stated_s1231_gain" value="${state.sep_stated_s1231_gain}"></label>
                <label><span data-i18n="view.s1366.label.charitable">Charitable contributions ($)</span>
                    <input type="number" step="1000" name="sep_stated_charitable" value="${state.sep_stated_charitable}"></label>
                <label><span data-i18n="view.s1366.label.int">Tax-exempt int income ($)</span>
                    <input type="number" step="1000" name="sep_stated_int_income" value="${state.sep_stated_int_income}"></label>
                <label><span data-i18n="view.s1366.label.days">Days owned</span>
                    <input type="number" step="1" name="days_owned" value="${state.days_owned}"></label>
                <label><span data-i18n="view.s1366.label.outside">Outside basis ($)</span>
                    <input type="number" step="10000" name="outside_basis" value="${state.outside_basis}"></label>
                <label><span data-i18n="view.s1366.label.at_risk">At-risk amount ($)</span>
                    <input type="number" step="10000" name="at_risk_amount" value="${state.at_risk_amount}"></label>
                <label><span data-i18n="view.s1366.label.passive">Passive shareholder (PAL applies)?</span>
                    <input type="checkbox" name="is_passive" ${state.is_passive ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1366.label.qbi">QBI § 199A eligible portion ($)</span>
                    <input type="number" step="1000" name="qbi_eligible_amount" value="${state.qbi_eligible_amount}"></label>
                <label><span data-i18n="view.s1366.label.marginal">Marginal rate %</span>
                    <input type="number" step="0.1" name="shareholder_marginal" value="${state.shareholder_marginal}"></label>
                <button class="primary" type="submit" data-i18n="view.s1366.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1366-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1366.h2.character">Character preservation</h2>
            <ul class="muted small">
                <li data-i18n="view.s1366.char.ltcg">Long-term capital gain stays LTCG (20%/15%/0%)</li>
                <li data-i18n="view.s1366.char.qdiv">Qualified dividends keep qualified character + 60-day holding</li>
                <li data-i18n="view.s1366.char.s1231">§ 1231 gain / loss: net at shareholder level → cap gain if net gain, ordinary if net loss</li>
                <li data-i18n="view.s1366.char.charitable">Charitable contributions: subject to shareholder's AGI limits (60%, 30%, 20%)</li>
                <li data-i18n="view.s1366.char.tax_exempt">Tax-exempt interest: keeps exempt character (raises basis without inclusion)</li>
                <li data-i18n="view.s1366.char.salt">SALT taxes on K-1: subject to $10K cap (BBA-PTET workaround)</li>
                <li data-i18n="view.s1366.char.niit">§ 1411 NIIT 3.8%: applies if shareholder is passive</li>
                <li data-i18n="view.s1366.char.dpad">§ 199A QBI 20% deduction for active trade or business income</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1366.h2.basis">Stock + debt basis (§ 1367)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1366.basis.increase">INCREASE basis: ordinary income + sep stated income + tax-exempt int + cap contributions</li>
                <li data-i18n="view.s1366.basis.decrease">DECREASE basis: distributions + losses + sep stated losses + non-deductible</li>
                <li data-i18n="view.s1366.basis.order">Order: increases first, then distributions, then losses</li>
                <li data-i18n="view.s1366.basis.stock_first">Stock basis used first for losses; then debt basis (from direct loans only)</li>
                <li data-i18n="view.s1366.basis.suspended">Loss in excess of basis SUSPENDED → carry forward indefinitely</li>
                <li data-i18n="view.s1366.basis.restored">Suspended loss claimed when basis restored (additional contribution / income)</li>
                <li data-i18n="view.s1366.basis.guarantee">Guarantee of S-corp debt does NOT create debt basis (need direct loan)</li>
                <li data-i18n="view.s1366.basis.restore_debt">Repaid debt basis: gain to extent basis restored if reduced</li>
            </ul>
        </div>
    `;
    document.getElementById('s1366-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.ownership_pct = Number(fd.get('ownership_pct')) || 0;
        state.ordinary_business_income = Number(fd.get('ordinary_business_income')) || 0;
        state.sep_stated_ltcg = Number(fd.get('sep_stated_ltcg')) || 0;
        state.sep_stated_qualified_div = Number(fd.get('sep_stated_qualified_div')) || 0;
        state.sep_stated_s1231_gain = Number(fd.get('sep_stated_s1231_gain')) || 0;
        state.sep_stated_charitable = Number(fd.get('sep_stated_charitable')) || 0;
        state.sep_stated_int_income = Number(fd.get('sep_stated_int_income')) || 0;
        state.days_owned = Number(fd.get('days_owned')) || 0;
        state.outside_basis = Number(fd.get('outside_basis')) || 0;
        state.at_risk_amount = Number(fd.get('at_risk_amount')) || 0;
        state.is_passive = !!fd.get('is_passive');
        state.qbi_eligible_amount = Number(fd.get('qbi_eligible_amount')) || 0;
        state.shareholder_marginal = Number(fd.get('shareholder_marginal')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1366-output');
    if (!el) return;
    const dailyFactor = state.days_owned / 365;
    const obiAllocated = state.ordinary_business_income * (state.ownership_pct / 100) * dailyFactor;
    const ltcgAllocated = state.sep_stated_ltcg * (state.ownership_pct / 100) * dailyFactor;
    const qdivAllocated = state.sep_stated_qualified_div * (state.ownership_pct / 100) * dailyFactor;
    const s1231Allocated = state.sep_stated_s1231_gain * (state.ownership_pct / 100) * dailyFactor;
    const taxExemptAllocated = state.sep_stated_int_income * (state.ownership_pct / 100) * dailyFactor;
    let lossFromOBI = obiAllocated < 0 ? Math.abs(obiAllocated) : 0;
    const allowedLoss = Math.min(lossFromOBI, state.outside_basis, state.at_risk_amount);
    const suspendedLoss = lossFromOBI - allowedLoss;
    const effectiveOBI = obiAllocated < 0 ? -allowedLoss : obiAllocated;
    const qbiDeduction = state.qbi_eligible_amount * 0.20;
    const taxOnOBI = effectiveOBI > 0 ? (effectiveOBI - qbiDeduction) * (state.shareholder_marginal / 100) : 0;
    const taxOnLTCG = ltcgAllocated * 0.20;
    const taxOnQDiv = qdivAllocated * 0.20;
    const taxOnS1231 = s1231Allocated > 0 ? s1231Allocated * 0.20 : s1231Allocated * (state.shareholder_marginal / 100);
    const niitOnPassive = state.is_passive ? (obiAllocated + ltcgAllocated + qdivAllocated) * 0.038 : 0;
    const totalTax = Math.max(0, taxOnOBI) + taxOnLTCG + taxOnQDiv + taxOnS1231 + niitOnPassive;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1366.h2.result">§ 1366 pass-through allocation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1366.card.daily_factor">Daily ownership factor</div>
                    <div class="value">${(dailyFactor * 100).toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1366.card.obi">OBI allocated</div>
                    <div class="value">$${obiAllocated.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${suspendedLoss > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s1366.card.suspended">Suspended loss</div>
                    <div class="value">$${suspendedLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1366.card.ltcg">LTCG allocated</div>
                    <div class="value">$${ltcgAllocated.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1366.card.qdiv">QDiv allocated</div>
                    <div class="value">$${qdivAllocated.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1366.card.qbi">QBI § 199A 20% deduction</div>
                    <div class="value">$${qbiDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1366.card.tax">Total tax due</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1366.card.tax_exempt">Tax-exempt int (basis only)</div>
                    <div class="value">$${taxExemptAllocated.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${suspendedLoss > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1366.suspended_note">
                    Suspended loss carries forward indefinitely until basis restored via additional capital
                    contribution OR future S-corp income. Consider loan-from-shareholder (direct, not guarantee)
                    to create debt basis and unlock loss.
                </p>
            ` : ''}
        </div>
    `;
}
