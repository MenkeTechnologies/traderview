// IRC § 6321 + § 6323 — Federal Tax Lien + Priority.
// Lien arises AUTOMATICALLY at assessment for tax + penalty + interest (NFTL not required).
// NFTL (Notice of Federal Tax Lien) PUBLIC RECORD priority vs other secured creditors.
// "Choateness" doctrine: lien attaches to ALL property + rights to property.
// § 6325 release / discharge / withdrawal / subordination options. Affects credit.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    assessed_tax: 0,
    penalty_interest: 0,
    has_nftl_filed: false,
    months_since_filing: 0,
    secured_creditors_existed_prior: 0,
    spouse_owns_50_pct: false,
    homestead_value: 0,
    homestead_outstanding_mortgage: 0,
    is_business_property: false,
};

export async function renderSection6321(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6321.h1.title">// § 6321 FEDERAL TAX LIEN</span></h1>
        <p class="muted small" data-i18n="view.s6321.hint.intro">
            Federal tax lien arises <strong>AUTOMATICALLY at assessment</strong> for tax + penalty
            + interest. Does NOT require NFTL filing. <strong>NFTL (Notice of Federal Tax Lien)
            sets PUBLIC priority</strong> vs other secured creditors. Attaches to <strong>ALL
            property + rights to property</strong> (Drye doctrine). Affects credit score until
            10-yr CSED expiration or release. <strong>§ 6325 options:</strong> release, discharge
            (specific property), withdrawal (false), subordination.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6321.h2.inputs">Inputs</h2>
            <form id="s6321-form" class="inline-form">
                <label><span data-i18n="view.s6321.label.tax">Assessed tax ($)</span>
                    <input type="number" step="0.01" name="assessed_tax" value="${state.assessed_tax}"></label>
                <label><span data-i18n="view.s6321.label.penalty">Penalty + interest ($)</span>
                    <input type="number" step="0.01" name="penalty_interest" value="${state.penalty_interest}"></label>
                <label><span data-i18n="view.s6321.label.nftl">NFTL filed?</span>
                    <input type="checkbox" name="has_nftl_filed" ${state.has_nftl_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6321.label.months">Months since filing</span>
                    <input type="number" step="1" name="months_since_filing" value="${state.months_since_filing}"></label>
                <label><span data-i18n="view.s6321.label.prior_secured">Prior secured creditors ($)</span>
                    <input type="number" step="0.01" name="secured_creditors_existed_prior" value="${state.secured_creditors_existed_prior}"></label>
                <label><span data-i18n="view.s6321.label.spouse">Spouse owns 50%+ jointly?</span>
                    <input type="checkbox" name="spouse_owns_50_pct" ${state.spouse_owns_50_pct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6321.label.homestead">Homestead value ($)</span>
                    <input type="number" step="0.01" name="homestead_value" value="${state.homestead_value}"></label>
                <label><span data-i18n="view.s6321.label.mortgage">Outstanding mortgage ($)</span>
                    <input type="number" step="0.01" name="homestead_outstanding_mortgage" value="${state.homestead_outstanding_mortgage}"></label>
                <label><span data-i18n="view.s6321.label.business">Business property?</span>
                    <input type="checkbox" name="is_business_property" ${state.is_business_property ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6321.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6321-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6321.h2.options">§ 6325 release / discharge options</h2>
            <ul class="muted small">
                <li data-i18n="view.s6321.opt.release">Release (§ 6325(a)): full payment OR CSED expired OR bond posted</li>
                <li data-i18n="view.s6321.opt.discharge">Discharge specific property (§ 6325(b)): pay equity attributable to that property</li>
                <li data-i18n="view.s6321.opt.subordination">Subordination (§ 6325(d)): IRS gives priority to another lender</li>
                <li data-i18n="view.s6321.opt.withdrawal">Withdrawal (§ 6323(j)): erroneous filing OR not in best interest OR direct debit agreement</li>
                <li data-i18n="view.s6321.opt.cdp">CDP hearing (§ 6320): right to challenge NFTL within 30 days of filing</li>
                <li data-i18n="view.s6321.opt.equivalent">Equivalent hearing: post-30-day window, less powerful</li>
                <li data-i18n="view.s6321.opt.fresh_start">"Fresh Start" 2011: NFTL threshold raised to $10k; withdrawal after full pay</li>
                <li data-i18n="view.s6321.opt.compromise">Offer-in-Compromise § 7122 settles debt for less + lien release</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6321.h2.priority">§ 6323 priority rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s6321.pri.first_in_time">First-in-time-first-in-right (compared to NFTL filing date)</li>
                <li data-i18n="view.s6321.pri.purchaser">"Purchaser" / "holder of security interest" before NFTL = priority</li>
                <li data-i18n="view.s6321.pri.45_day">45-day window after NFTL: existing secured creditor still has priority</li>
                <li data-i18n="view.s6321.pri.super_priority">"Super-priority" exceptions: § 6323(b) — wages, attorney fees, casualty insurance, etc.</li>
                <li data-i18n="view.s6321.pri.mechanics">Mechanics lien + materialman lien generally retain priority</li>
                <li data-i18n="view.s6321.pri.real_property">Real property: file with county recorder where situated</li>
                <li data-i18n="view.s6321.pri.personal">Personal property: file with state designation (UCC-like)</li>
            </ul>
        </div>
    `;
    document.getElementById('s6321-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.assessed_tax = Number(fd.get('assessed_tax')) || 0;
        state.penalty_interest = Number(fd.get('penalty_interest')) || 0;
        state.has_nftl_filed = !!fd.get('has_nftl_filed');
        state.months_since_filing = Number(fd.get('months_since_filing')) || 0;
        state.secured_creditors_existed_prior = Number(fd.get('secured_creditors_existed_prior')) || 0;
        state.spouse_owns_50_pct = !!fd.get('spouse_owns_50_pct');
        state.homestead_value = Number(fd.get('homestead_value')) || 0;
        state.homestead_outstanding_mortgage = Number(fd.get('homestead_outstanding_mortgage')) || 0;
        state.is_business_property = !!fd.get('is_business_property');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6321-output');
    if (!el) return;
    const totalDebt = state.assessed_tax + state.penalty_interest;
    const irsAttachable = state.spouse_owns_50_pct ? totalDebt * 0.5 : totalDebt;
    const homesteadEquity = Math.max(0, state.homestead_value - state.homestead_outstanding_mortgage);
    const irsEquityClaim = Math.max(0, homesteadEquity - state.secured_creditors_existed_prior);
    const collectibleNow = Math.min(irsAttachable, irsEquityClaim);
    const remaining10yrCsed = state.has_nftl_filed ? Math.max(0, 120 - state.months_since_filing) : 120;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6321.h2.result">Lien analysis</h2>
            <div class="cards">
                <div class="card neg">
                    <div class="label" data-i18n="view.s6321.card.total">Total lien amount</div>
                    <div class="value">$${totalDebt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6321.card.attachable">IRS attachable</div>
                    <div class="value">$${irsAttachable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6321.card.equity">Homestead equity</div>
                    <div class="value">$${homesteadEquity.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6321.card.collectible">IRS collectible now (homestead)</div>
                    <div class="value">$${collectibleNow.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${state.has_nftl_filed ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6321.card.nftl">NFTL filed?</div>
                    <div class="value">${state.has_nftl_filed ? esc(t('view.s6321.status.yes')) : esc(t('view.s6321.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6321.card.csed_remaining">CSED months remaining</div>
                    <div class="value">${remaining10yrCsed}</div>
                </div>
            </div>
        </div>
    `;
}
