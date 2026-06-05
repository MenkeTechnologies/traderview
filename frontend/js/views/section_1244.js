// IRC § 1244 Ordinary Loss on Small Business Stock.
// Up to $50,000 single / $100,000 MFJ of loss converted from capital to ORDINARY.
// Excess capital loss still subject to $3,000/yr offset against ordinary.
// Requirements: domestic corp, $1M aggregate cap raised (cumulative), shareholder
// is original issuee (not transferee), corp earns < 50% passive income last 5 yrs.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const ORDINARY_CAP_SINGLE = 50_000;
const ORDINARY_CAP_MFJ = 100_000;
const CORP_AGGREGATE_LIMIT = 1_000_000;
const CAPITAL_LOSS_ANNUAL_OFFSET = 3_000;

let state = {
    realized_loss: 0,
    cost_basis: 0,
    sale_price: 0,
    filing_status: 'single',
    is_original_issuee: true,
    corp_aggregate_capital: 0,
    corp_passive_income_pct: 0,
    is_domestic: true,
    marginal_rate: 0.35,
    ltcg_rate: 0.20,
};

export async function renderSection1244(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1244.h1.title">// § 1244 ORDINARY LOSS ON SBS</span></h1>
        <p class="muted small" data-i18n="view.s1244.hint.intro">
            Convert capital loss to <strong>ORDINARY loss</strong> up to <strong>$50k single /
            $100k MFJ</strong> when small business stock goes south. Stock must be in a domestic
            corp with $1M aggregate capital raised (cumulative). You must be the ORIGINAL ISSUEE,
            not a transferee. Major rescue for failed startup investors.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1244.h2.qualification">Qualification requirements</h2>
            <ul class="muted small">
                <li data-i18n="view.s1244.qual.domestic">Domestic C-corp or S-corp (not LLC, not foreign)</li>
                <li data-i18n="view.s1244.qual.aggregate">Corp aggregate capital raised at issuance ≤ $1,000,000</li>
                <li data-i18n="view.s1244.qual.passive">&lt; 50% of gross receipts from passive sources (royalties, rents, dividends, interest, capital gain) over preceding 5 years</li>
                <li data-i18n="view.s1244.qual.issuee">You must be the ORIGINAL ISSUEE (acquired in exchange for money or property, NOT services)</li>
                <li data-i18n="view.s1244.qual.individual">Only individuals + partnerships qualify (not corporations, not trusts)</li>
                <li data-i18n="view.s1244.qual.no_swap">NOT acquired via tax-free reorg / contribution</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1244.h2.inputs">Inputs</h2>
            <form id="s1244-form" class="inline-form">
                <label><span data-i18n="view.s1244.label.cost_basis">Your cost basis ($)</span>
                    <input type="number" step="0.01" name="cost_basis" value="${state.cost_basis}"></label>
                <label><span data-i18n="view.s1244.label.sale_price">Sale or worthless price ($)</span>
                    <input type="number" step="0.01" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s1244.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1244.label.is_original">Original issuee?</span>
                    <input type="checkbox" name="is_original_issuee" ${state.is_original_issuee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1244.label.corp_capital">Corp aggregate capital raised at issuance ($)</span>
                    <input type="number" step="0.01" name="corp_aggregate_capital" value="${state.corp_aggregate_capital}"></label>
                <label><span data-i18n="view.s1244.label.passive_pct">Corp 5-yr passive income %</span>
                    <input type="number" step="0.01" name="corp_passive_income_pct" value="${state.corp_passive_income_pct}"></label>
                <label><span data-i18n="view.s1244.label.is_domestic">Domestic corp?</span>
                    <input type="checkbox" name="is_domestic" ${state.is_domestic ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1244.label.marginal">Marginal ordinary rate %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s1244.label.ltcg_rate">LTCG rate %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s1244.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1244-output"></div>
    `;
    document.getElementById('s1244-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.cost_basis = Number(fd.get('cost_basis')) || 0;
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.filing_status = fd.get('filing_status');
        state.is_original_issuee = !!fd.get('is_original_issuee');
        state.corp_aggregate_capital = Number(fd.get('corp_aggregate_capital')) || 0;
        state.corp_passive_income_pct = Number(fd.get('corp_passive_income_pct')) || 0;
        state.is_domestic = !!fd.get('is_domestic');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.35;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1244-output');
    if (!el) return;
    const totalLoss = Math.max(0, state.cost_basis - state.sale_price);
    const passes_domestic = state.is_domestic;
    const passes_capital = state.corp_aggregate_capital <= CORP_AGGREGATE_LIMIT;
    const passes_passive = state.corp_passive_income_pct < 0.50;
    const passes_issuee = state.is_original_issuee;
    const qualifies = passes_domestic && passes_capital && passes_passive && passes_issuee;
    const ordinaryCap = state.filing_status === 'mfj' ? ORDINARY_CAP_MFJ : ORDINARY_CAP_SINGLE;
    const ordinaryLoss = qualifies ? Math.min(totalLoss, ordinaryCap) : 0;
    const remainingCapLoss = totalLoss - ordinaryLoss;
    const ordinarySavings = ordinaryLoss * state.marginal_rate;
    const capYr1Savings = Math.min(remainingCapLoss, CAPITAL_LOSS_ANNUAL_OFFSET) * state.marginal_rate;
    const capYearsToUse = remainingCapLoss > 0 ? Math.ceil(remainingCapLoss / CAPITAL_LOSS_ANNUAL_OFFSET) : 0;
    const ifAllOrdinary = qualifies ? ordinarySavings + (remainingCapLoss * state.marginal_rate) : 0;
    const ifAllCap = totalLoss * state.ltcg_rate;
    const benefit = qualifies ? (ifAllOrdinary - ifAllCap) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1244.h2.result">Calculation</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1244.card.qualifies">§ 1244 qualifies?</div>
                    <div class="value">${qualifies ? esc(t('view.s1244.status.yes')) : esc(t('view.s1244.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1244.card.total_loss">Total loss</div>
                    <div class="value">$${totalLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1244.card.ordinary_loss">Ordinary loss (§ 1244)</div>
                    <div class="value">$${ordinaryLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1244.card.cap_loss">Remaining capital loss</div>
                    <div class="value">$${remainingCapLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1244.card.year1_savings">Year-1 tax savings</div>
                    <div class="value">$${(ordinarySavings + capYr1Savings).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1244.card.cap_years">Cap loss years to absorb</div>
                    <div class="value">${capYearsToUse}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1244.card.benefit">§ 1244 benefit vs. cap</div>
                    <div class="value">$${benefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!qualifies ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1244.warning.fails">
                    Does NOT qualify. Loss is capital, subject to $3,000/yr offset.
                    ${!passes_domestic ? 'Not domestic. ' : ''}${!passes_capital ? `Corp raised > $1M. ` : ''}${!passes_passive ? 'Corp ≥50% passive. ' : ''}${!passes_issuee ? 'Not original issuee. ' : ''}
                </p>
            ` : ''}
        </div>
    `;
}
