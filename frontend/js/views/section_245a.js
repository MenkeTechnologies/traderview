// IRC § 245A — Participation Exemption (100% DRD for Foreign-Source Dividends).
// 100% deduction for foreign-source portion of dividends from 10%-owned foreign corps.
// One-year holding period required (45-day for preferred).
// Hybrid dividend rule: § 245A(e) denies deduction if foreign payor deducted it.
// § 1248: stock sale gain recharacterizes E&P as dividend → eligible for § 245A.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    dividend_received: 0,
    foreign_source_pct: 100,
    us_ownership_pct: 0,
    holding_period_days: 0,
    hybrid_dividend: false,
    is_preferred: false,
    cfc_status: false,
    s1248_recharacterized: false,
    foreign_tax_paid: 0,
};

export async function renderSection245A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s245A.h1.title">// § 245A 100% DRD (Participation)</span></h1>
        <p class="muted small" data-i18n="view.s245A.hint.intro">
            <strong>100% deduction</strong> for foreign-source portion of dividends from 10%-owned foreign
            corp ("specified 10% owned"). <strong>Holding period:</strong> 365 / 731 days (45 / 90 for preferred).
            <strong>Hybrid dividend rule (§ 245A(e)):</strong> deduction DENIED if foreign payor deducted it.
            <strong>§ 1248 gain</strong> on CFC stock sale recharacterized to E&P → eligible for § 245A.
            <strong>Net effect:</strong> US C-corps largely exempt from tax on actual foreign dividends.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s245A.h2.inputs">Inputs</h2>
            <form id="s245A-form" class="inline-form">
                <label><span data-i18n="view.s245A.label.dividend">Dividend received ($)</span>
                    <input type="number" step="1000" name="dividend_received" value="${state.dividend_received}"></label>
                <label><span data-i18n="view.s245A.label.foreign_pct">Foreign-source % of dividend</span>
                    <input type="number" step="0.1" name="foreign_source_pct" value="${state.foreign_source_pct}"></label>
                <label><span data-i18n="view.s245A.label.ownership">US shareholder ownership %</span>
                    <input type="number" step="0.1" name="us_ownership_pct" value="${state.us_ownership_pct}"></label>
                <label><span data-i18n="view.s245A.label.holding">Holding period days</span>
                    <input type="number" step="1" name="holding_period_days" value="${state.holding_period_days}"></label>
                <label><span data-i18n="view.s245A.label.hybrid">Hybrid dividend?</span>
                    <input type="checkbox" name="hybrid_dividend" ${state.hybrid_dividend ? 'checked' : ''}></label>
                <label><span data-i18n="view.s245A.label.preferred">Preferred stock?</span>
                    <input type="checkbox" name="is_preferred" ${state.is_preferred ? 'checked' : ''}></label>
                <label><span data-i18n="view.s245A.label.cfc">CFC stock?</span>
                    <input type="checkbox" name="cfc_status" ${state.cfc_status ? 'checked' : ''}></label>
                <label><span data-i18n="view.s245A.label.s1248">§ 1248 recharacterized stock sale gain?</span>
                    <input type="checkbox" name="s1248_recharacterized" ${state.s1248_recharacterized ? 'checked' : ''}></label>
                <label><span data-i18n="view.s245A.label.foreign_tax">Foreign tax withheld ($)</span>
                    <input type="number" step="100" name="foreign_tax_paid" value="${state.foreign_tax_paid}"></label>
                <button class="primary" type="submit" data-i18n="view.s245A.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s245A-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s245A.h2.requirements">§ 245A requirements</h2>
            <ul class="muted small">
                <li data-i18n="view.s245A.req.us_corp">Recipient must be US C-corp (no S-corp, no individual)</li>
                <li data-i18n="view.s245A.req.specified10">"Specified 10% owned" foreign corp: 10%+ by vote OR value</li>
                <li data-i18n="view.s245A.req.holding">Holding period: 365 days within 731-day window (45 / 90 days for preferred)</li>
                <li data-i18n="view.s245A.req.foreign_source">Foreign-source portion only (US-source portion taxed)</li>
                <li data-i18n="view.s245A.req.not_pfic">Foreign payor cannot be PFIC for any year of US shareholder ownership</li>
                <li data-i18n="view.s245A.req.no_hybrid">Not a hybrid dividend (would create double non-tax)</li>
                <li data-i18n="view.s245A.req.no_ftc">No § 901 FTC + no § 902 indirect credit on excluded portion (logical — no income to credit against)</li>
                <li data-i18n="view.s245A.req.no_deduction">Expenses allocable to deductible dividend NOT deductible (no whipsaw)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s245A.h2.s1248">§ 1248 Interaction</h2>
            <ul class="muted small">
                <li data-i18n="view.s245A.s1248.basic">CFC stock sale: gain up to allocated E&P recharacterized as DIVIDEND under § 1248</li>
                <li data-i18n="view.s245A.s1248.deduction">Recharacterized portion eligible for § 245A 100% DRD (huge planning win)</li>
                <li data-i18n="view.s245A.s1248.requires">Still requires holding period + foreign source + not hybrid</li>
                <li data-i18n="view.s245A.s1248.deconsolidation">Trumps gain on CFC sale → enables tax-free repatriation via sale</li>
                <li data-i18n="view.s245A.s1248.gp_election">Gainproration § 1248(a) plus § 245A combo</li>
                <li data-i18n="view.s245A.s1248.ordering">Order: § 1248 recharacterization FIRST, then § 245A applied</li>
            </ul>
        </div>
    `;
    document.getElementById('s245A-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.dividend_received = Number(fd.get('dividend_received')) || 0;
        state.foreign_source_pct = Number(fd.get('foreign_source_pct')) || 0;
        state.us_ownership_pct = Number(fd.get('us_ownership_pct')) || 0;
        state.holding_period_days = Number(fd.get('holding_period_days')) || 0;
        state.hybrid_dividend = !!fd.get('hybrid_dividend');
        state.is_preferred = !!fd.get('is_preferred');
        state.cfc_status = !!fd.get('cfc_status');
        state.s1248_recharacterized = !!fd.get('s1248_recharacterized');
        state.foreign_tax_paid = Number(fd.get('foreign_tax_paid')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s245A-output');
    if (!el) return;
    const ownershipMet = state.us_ownership_pct >= 10;
    const requiredHolding = state.is_preferred ? 90 : 365;
    const holdingMet = state.holding_period_days >= requiredHolding;
    const notHybrid = !state.hybrid_dividend;
    const eligible = ownershipMet && holdingMet && notHybrid;
    const foreignSourcePortion = state.dividend_received * (state.foreign_source_pct / 100);
    const drd = eligible ? foreignSourcePortion : 0;
    const taxableDividend = state.dividend_received - drd;
    const taxOwed = taxableDividend * 0.21;
    const taxSavings = drd * 0.21;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s245A.h2.result">§ 245A computation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s245A.card.eligible">Eligible for 100% DRD?</div>
                    <div class="value">${eligible ? esc(t('view.s245A.status.yes')) : esc(t('view.s245A.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s245A.card.ownership_test">Ownership ≥ 10%</div>
                    <div class="value">${ownershipMet ? esc(t('view.s245A.status.yes')) : esc(t('view.s245A.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s245A.card.holding_test">Holding ≥ ${requiredHolding}d</div>
                    <div class="value">${holdingMet ? esc(t('view.s245A.status.yes')) : esc(t('view.s245A.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s245A.card.foreign_portion">Foreign-source portion</div>
                    <div class="value">$${foreignSourcePortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s245A.card.drd">§ 245A DRD</div>
                    <div class="value">$${drd.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s245A.card.taxable">Taxable dividend</div>
                    <div class="value">$${taxableDividend.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s245A.card.tax">US tax (21%)</div>
                    <div class="value">$${taxOwed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s245A.card.savings">Tax savings via DRD</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.s1248_recharacterized && eligible ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s245A.s1248_note">
                    § 1248 recharacterized gain captured under § 245A — full DRD on E&P portion. Net result:
                    tax-free repatriation via CFC sale. Coordinate with § 367 outbound transfer rules.
                </p>
            ` : ''}
        </div>
    `;
}
