// IRC § 305 — Stock Dividends + Deemed Dividends.
// General rule § 305(a): stock dividend is TAX-FREE — just split ownership pro-rata.
// § 305(b) EXCEPTIONS: 5 categories trigger ORDINARY DIVIDEND treatment.
// § 305(c) DEEMED dividends: Section 305(c) applies to conversion ratio adjustments, accrued PIK on preferred.
// Holding period tacks; basis allocated by relative FMV.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    distribution_type: 'pro_rata_common',
    fmv_of_stock_distributed: 0,
    fmv_of_stock_before: 0,
    accumulated_ep: 0,
    cash_or_property_option: false,
    disproportionate_dist: false,
    common_and_preferred: false,
    preferred_dividends_pik: false,
    conversion_ratio_adjustment: false,
    accrued_dividend_arrearages: false,
    shareholder_basis_orig: 0,
};

export async function renderSection305(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s305.h1.title">// § 305 STOCK DIVIDENDS</span></h1>
        <p class="muted small" data-i18n="view.s305.hint.intro">
            <strong>§ 305(a):</strong> Stock dividend = TAX-FREE; just splits ownership pro-rata. Holding
            period tacks; basis allocated by relative FMV. <strong>§ 305(b) EXCEPTIONS</strong> (5 categories)
            trigger ORDINARY DIVIDEND treatment: (1) cash or stock option, (2) disproportionate distribution,
            (3) common + preferred to different shareholders, (4) preferred-stock-on-preferred, (5) convertible
            preferred. <strong>§ 305(c):</strong> DEEMED dividends — conversion ratio adjustments, accrued
            PIK on preferred, arrearages.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s305.h2.inputs">Inputs</h2>
            <form id="s305-form" class="inline-form">
                <label><span data-i18n="view.s305.label.type">Distribution type</span>
                    <select name="distribution_type">
                        <option value="pro_rata_common" ${state.distribution_type === 'pro_rata_common' ? 'selected' : ''}>Pro-rata common on common</option>
                        <option value="stock_split" ${state.distribution_type === 'stock_split' ? 'selected' : ''}>Stock split</option>
                        <option value="optional_cash_stock" ${state.distribution_type === 'optional_cash_stock' ? 'selected' : ''}>Optional cash or stock</option>
                        <option value="disproportionate" ${state.distribution_type === 'disproportionate' ? 'selected' : ''}>Disproportionate (some cash, some stock)</option>
                        <option value="common_and_preferred" ${state.distribution_type === 'common_and_preferred' ? 'selected' : ''}>Common + preferred to different holders</option>
                        <option value="preferred_on_preferred" ${state.distribution_type === 'preferred_on_preferred' ? 'selected' : ''}>Preferred on preferred</option>
                        <option value="convertible_preferred" ${state.distribution_type === 'convertible_preferred' ? 'selected' : ''}>Convertible preferred</option>
                    </select>
                </label>
                <label><span data-i18n="view.s305.label.fmv_dist">FMV of stock distributed ($)</span>
                    <input type="number" step="10000" name="fmv_of_stock_distributed" value="${state.fmv_of_stock_distributed}"></label>
                <label><span data-i18n="view.s305.label.fmv_before">FMV of pre-dist stock ($)</span>
                    <input type="number" step="10000" name="fmv_of_stock_before" value="${state.fmv_of_stock_before}"></label>
                <label><span data-i18n="view.s305.label.ep">Accumulated E&P ($)</span>
                    <input type="number" step="10000" name="accumulated_ep" value="${state.accumulated_ep}"></label>
                <label><span data-i18n="view.s305.label.option">Cash or property option available?</span>
                    <input type="checkbox" name="cash_or_property_option" ${state.cash_or_property_option ? 'checked' : ''}></label>
                <label><span data-i18n="view.s305.label.dispro">Disproportionate distribution?</span>
                    <input type="checkbox" name="disproportionate_dist" ${state.disproportionate_dist ? 'checked' : ''}></label>
                <label><span data-i18n="view.s305.label.both">Common + preferred to different holders?</span>
                    <input type="checkbox" name="common_and_preferred" ${state.common_and_preferred ? 'checked' : ''}></label>
                <label><span data-i18n="view.s305.label.pik">Preferred dividends PIK (paid in kind)?</span>
                    <input type="checkbox" name="preferred_dividends_pik" ${state.preferred_dividends_pik ? 'checked' : ''}></label>
                <label><span data-i18n="view.s305.label.conv_adj">Conversion ratio adjustment?</span>
                    <input type="checkbox" name="conversion_ratio_adjustment" ${state.conversion_ratio_adjustment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s305.label.arrearages">Accrued dividend arrearages on preferred?</span>
                    <input type="checkbox" name="accrued_dividend_arrearages" ${state.accrued_dividend_arrearages ? 'checked' : ''}></label>
                <label><span data-i18n="view.s305.label.basis">Shareholder original basis ($)</span>
                    <input type="number" step="10000" name="shareholder_basis_orig" value="${state.shareholder_basis_orig}"></label>
                <button class="primary" type="submit" data-i18n="view.s305.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s305-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s305.h2.b_exceptions">§ 305(b) — five exceptions triggering dividend</h2>
            <ol class="muted small">
                <li data-i18n="view.s305.b1">Distribution in lieu of money: shareholder option of cash OR stock</li>
                <li data-i18n="view.s305.b2">Disproportionate distribution: some shareholders receive cash, others stock — increased proportionate interest</li>
                <li data-i18n="view.s305.b3">Common AND preferred distributions: common to some shareholders + preferred to others</li>
                <li data-i18n="view.s305.b4">Preferred stock on preferred: distribution of preferred on preferred (except antidilution adj)</li>
                <li data-i18n="view.s305.b5">Convertible preferred: where any shareholder will have increased proportionate interest</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s305.h2.c_deemed">§ 305(c) — deemed dividends</h2>
            <ul class="muted small">
                <li data-i18n="view.s305.c.conversion">Conversion ratio adjustment: increases conversion advantage of one class vs others</li>
                <li data-i18n="view.s305.c.pik_preferred">PIK preferred: accrual of dividends paid in additional preferred = deemed dist</li>
                <li data-i18n="view.s305.c.arrearages">Accrued / suspended dividend arrearages on preferred: treated as deemed dividends in some cases</li>
                <li data-i18n="view.s305.c.anti_dilution">Anti-dilution adjustments: safe harbor if proportionate to ALL stock</li>
                <li data-i18n="view.s305.c.warrants">Warrants + options: § 305(d) treats holders as shareholders for these rules</li>
                <li data-i18n="view.s305.c.amount">Amount: FMV of increased interest (use binomial / Black-Scholes for convertibles)</li>
                <li data-i18n="view.s305.c.timing">Timing: economic accrual — not just legal payment date</li>
                <li data-i18n="view.s305.c.reporting">Form 1099-DIV reporting + Form 8937 corporate action issuer reporting</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s305.h2.basis_allocation">Basis allocation (§ 307)</h2>
            <ul class="muted small">
                <li data-i18n="view.s305.basis.relative">Allocate basis between old + new stock by RELATIVE FMV on distribution date</li>
                <li data-i18n="view.s305.basis.holding">Holding period of new stock TACKS to old stock</li>
                <li data-i18n="view.s305.basis.example">Example: $10K basis on 100 shares; 1-for-1 stock split; $5K basis on each lot</li>
                <li data-i18n="view.s305.basis.preferred_common">Preferred + common: allocate by FMV ratio (preferred at par or market)</li>
                <li data-i18n="view.s305.basis.tax_dividend">If § 305(b) makes it taxable: basis = FMV at distribution (full step-up)</li>
                <li data-i18n="view.s305.basis.short_term">Cash in lieu of fractional shares: reported on 1099-B as sale; partial gain</li>
                <li data-i18n="view.s305.basis.s306">§ 306 'tainted stock': bailout-prevention rule on later sale of preferred</li>
                <li data-i18n="view.s305.basis.recordkeeping">Form 8937 + 1099-DIV box reporting for issuer + broker</li>
            </ul>
        </div>
    `;
    document.getElementById('s305-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.distribution_type = fd.get('distribution_type');
        state.fmv_of_stock_distributed = Number(fd.get('fmv_of_stock_distributed')) || 0;
        state.fmv_of_stock_before = Number(fd.get('fmv_of_stock_before')) || 0;
        state.accumulated_ep = Number(fd.get('accumulated_ep')) || 0;
        state.cash_or_property_option = !!fd.get('cash_or_property_option');
        state.disproportionate_dist = !!fd.get('disproportionate_dist');
        state.common_and_preferred = !!fd.get('common_and_preferred');
        state.preferred_dividends_pik = !!fd.get('preferred_dividends_pik');
        state.conversion_ratio_adjustment = !!fd.get('conversion_ratio_adjustment');
        state.accrued_dividend_arrearages = !!fd.get('accrued_dividend_arrearages');
        state.shareholder_basis_orig = Number(fd.get('shareholder_basis_orig')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s305-output');
    if (!el) return;
    const triggersB = state.cash_or_property_option || state.disproportionate_dist || state.common_and_preferred || state.distribution_type === 'preferred_on_preferred' || state.distribution_type === 'convertible_preferred';
    const triggersC = state.preferred_dividends_pik || state.conversion_ratio_adjustment || state.accrued_dividend_arrearages;
    const triggersDividend = triggersB || triggersC;
    const dividendAmt = triggersDividend ? Math.min(state.fmv_of_stock_distributed, state.accumulated_ep) : 0;
    const dividendTax = dividendAmt * 0.20;
    const allocatedBasisOld = state.fmv_of_stock_before > 0 ? state.shareholder_basis_orig * state.fmv_of_stock_before / (state.fmv_of_stock_before + state.fmv_of_stock_distributed) : state.shareholder_basis_orig;
    const allocatedBasisNew = triggersDividend ? state.fmv_of_stock_distributed : state.shareholder_basis_orig - allocatedBasisOld;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s305.h2.result">§ 305 outcome</h2>
            <div class="cards">
                <div class="card ${triggersDividend ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s305.card.taxable">Triggers dividend?</div>
                    <div class="value">${triggersDividend ? esc(t('view.s305.status.yes')) : esc(t('view.s305.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s305.card.b_excep">§ 305(b) triggered?</div>
                    <div class="value">${triggersB ? esc(t('view.s305.status.yes')) : esc(t('view.s305.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s305.card.c_deemed">§ 305(c) triggered?</div>
                    <div class="value">${triggersC ? esc(t('view.s305.status.yes')) : esc(t('view.s305.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s305.card.dividend">Dividend recognized</div>
                    <div class="value">$${dividendAmt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s305.card.tax">Dividend tax (20%)</div>
                    <div class="value">$${dividendTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s305.card.basis_old">Allocated basis OLD</div>
                    <div class="value">$${allocatedBasisOld.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s305.card.basis_new">Allocated basis NEW</div>
                    <div class="value">$${allocatedBasisNew.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${triggersC ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s305.c_note">
                    § 305(c) deemed dividend triggered. PIK preferred accruals + conversion ratio adjustments
                    cause economic-substance taxation even without legal payment. Apply Form 8937 reporting
                    + 1099-DIV broker-level adjustment.
                </p>
            ` : ''}
        </div>
    `;
}
