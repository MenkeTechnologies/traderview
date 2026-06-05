// IRC § 871(m) — Dividend Equivalent Payment Withholding on Total Return Swaps.
// Foreign person receiving dividend-equivalent payment on US equity-linked instrument
// (TRS, swap, options, securities lending) subject to 30% withholding (or treaty rate).
// Delta-1 instruments + non-delta-1 with > 0.80 delta = "specified equity-linked instruments".
// QDD (Qualified Derivatives Dealer) regime simplifies. Notice 2024-44 latest guidance.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const DELTA_THRESHOLD = 0.80;
const DEFAULT_WITHHOLDING = 0.30;

let state = {
    is_foreign_counterparty: true,
    treaty_country: '',
    treaty_rate: 0.30,
    notional_value: 0,
    delta: 0,
    expected_dividend_yield_annual: 0,
    holding_years: 1,
    is_qdd: false,
    is_delta_one: false,
    instrument_type: 'total_return_swap',
};

const TREATY_RATES = {
    'UK': 0.15, 'Canada': 0.15, 'Germany': 0.15, 'France': 0.15, 'Japan': 0.10,
    'Netherlands': 0.15, 'Switzerland': 0.15, 'Australia': 0.15, 'Ireland': 0.15,
    'India': 0.15, 'Italy': 0.15, 'Spain': 0.15, 'Mexico': 0.10, 'Brazil': 0.15,
    'Singapore': 0.30, 'Hong Kong': 0.30,
};

export async function renderSection871m(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s871m.h1.title">// § 871(m) DIVIDEND EQUIVALENT WITHHOLDING</span></h1>
        <p class="muted small" data-i18n="view.s871m.hint.intro">
            Foreign person receiving "<strong>dividend-equivalent payment</strong>" on US-equity-linked
            instrument (TRS, swap, options, securities lending) subject to <strong>30% withholding</strong>
            (or treaty rate). <strong>Delta-1 instruments</strong> always covered.
            <strong>Non-delta-1 with delta &gt; 0.80</strong> = "specified equity-linked instruments".
            <strong>QDD regime</strong> for qualified dealers. Notice 2024-44 + Phase-in regs ongoing.
            <strong>Section 305(c)</strong> deemed distributions on convertibles also apply.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s871m.h2.inputs">Inputs</h2>
            <form id="s871m-form" class="inline-form">
                <label><span data-i18n="view.s871m.label.foreign">Foreign counterparty?</span>
                    <input type="checkbox" name="is_foreign_counterparty" ${state.is_foreign_counterparty ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871m.label.treaty">Treaty country</span>
                    <select name="treaty_country">
                        <option value="">None / non-treaty</option>
                        ${Object.keys(TREATY_RATES).map(c => `<option value="${c}" ${state.treaty_country === c ? 'selected' : ''}>${c}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.s871m.label.notional">Notional value ($)</span>
                    <input type="number" step="0.01" name="notional_value" value="${state.notional_value}"></label>
                <label><span data-i18n="view.s871m.label.delta">Delta</span>
                    <input type="number" step="0.01" min="0" max="1" name="delta" value="${state.delta}"></label>
                <label><span data-i18n="view.s871m.label.dividend">Expected dividend yield annual</span>
                    <input type="number" step="0.001" name="expected_dividend_yield_annual" value="${state.expected_dividend_yield_annual}"></label>
                <label><span data-i18n="view.s871m.label.years">Holding years</span>
                    <input type="number" step="0.25" name="holding_years" value="${state.holding_years}"></label>
                <label><span data-i18n="view.s871m.label.qdd">QDD-qualified dealer?</span>
                    <input type="checkbox" name="is_qdd" ${state.is_qdd ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871m.label.delta_one">Delta-1 instrument?</span>
                    <input type="checkbox" name="is_delta_one" ${state.is_delta_one ? 'checked' : ''}></label>
                <label><span data-i18n="view.s871m.label.instrument">Instrument type</span>
                    <select name="instrument_type">
                        <option value="total_return_swap">Total Return Swap (TRS)</option>
                        <option value="single_stock_future">Single Stock Future</option>
                        <option value="forward">Forward</option>
                        <option value="option">Option</option>
                        <option value="convertible">Convertible (305(c))</option>
                        <option value="securities_lending">Securities lending</option>
                        <option value="structured_note">Structured note</option>
                    </select>
                </label>
                <button class="primary" type="submit" data-i18n="view.s871m.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s871m-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871m.h2.delta">Delta-1 vs Non-delta-1</h2>
            <ul class="muted small">
                <li data-i18n="view.s871m.delta.delta_one">Delta-1 = 1:1 equity exposure (TRS, single-stock future, forwards)</li>
                <li data-i18n="view.s871m.delta.non_delta_one">Non-delta-1 with delta &gt; 0.80 at issuance = covered</li>
                <li data-i18n="view.s871m.delta.combined">Combined instruments with related counterparties: aggregated for delta test</li>
                <li data-i18n="view.s871m.delta.deemed_div">Deemed dividend payment = notional × dividend yield × time</li>
                <li data-i18n="view.s871m.delta.qualified_index">Qualified index (broad-market): generally outside § 871(m)</li>
                <li data-i18n="view.s871m.delta.ad_listed">ADR / listed company: equity-linked tests apply</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s871m.h2.qdd_regime">QDD Regime</h2>
            <p class="muted small" data-i18n="view.s871m.qdd.body">
                Qualified Derivatives Dealer (QDD) — typically broker-dealer or affiliate — assumes
                primary withholding responsibility on derivatives positions. Eliminates cascading
                withholding through chain of intermediaries. Requires QI agreement + W-8IMY +
                annual compliance. Notice 2024-44 extended phase-in transition rules.
            </p>
        </div>
    `;
    document.getElementById('s871m-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_foreign_counterparty = !!fd.get('is_foreign_counterparty');
        state.treaty_country = fd.get('treaty_country') || '';
        state.treaty_rate = TREATY_RATES[state.treaty_country] || DEFAULT_WITHHOLDING;
        state.notional_value = Number(fd.get('notional_value')) || 0;
        state.delta = Number(fd.get('delta')) || 0;
        state.expected_dividend_yield_annual = Number(fd.get('expected_dividend_yield_annual')) || 0;
        state.holding_years = Number(fd.get('holding_years')) || 1;
        state.is_qdd = !!fd.get('is_qdd');
        state.is_delta_one = !!fd.get('is_delta_one');
        state.instrument_type = fd.get('instrument_type');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s871m-output');
    if (!el) return;
    const covered = state.is_foreign_counterparty && (state.is_delta_one || state.delta >= DELTA_THRESHOLD);
    const annualDividendEquivalent = state.notional_value * state.expected_dividend_yield_annual;
    const totalDividendEquivalent = annualDividendEquivalent * state.holding_years;
    const applicableRate = state.is_qdd ? 0 : (state.treaty_rate || DEFAULT_WITHHOLDING);
    const withholdingPerYear = covered ? annualDividendEquivalent * applicableRate : 0;
    const totalWithholding = withholdingPerYear * state.holding_years;
    const noTreatyWithholding = covered ? totalDividendEquivalent * DEFAULT_WITHHOLDING : 0;
    const treatySavings = noTreatyWithholding - totalWithholding;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s871m.h2.result">§ 871(m) outcome</h2>
            <div class="cards">
                <div class="card ${covered ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s871m.card.covered">Covered by § 871(m)?</div>
                    <div class="value">${covered ? esc(t('view.s871m.status.yes')) : esc(t('view.s871m.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s871m.card.delta">Effective delta</div>
                    <div class="value">${state.is_delta_one ? '1.00' : state.delta.toFixed(2)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s871m.card.dividend">Annual dividend-equivalent</div>
                    <div class="value">$${annualDividendEquivalent.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s871m.card.total_dividend">Total over holding</div>
                    <div class="value">$${totalDividendEquivalent.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s871m.card.rate">Applicable rate</div>
                    <div class="value">${(applicableRate * 100).toFixed(0)}%</div>
                </div>
                <div class="card ${withholdingPerYear > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s871m.card.annual_with">Annual withholding</div>
                    <div class="value">$${withholdingPerYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalWithholding > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s871m.card.total_with">Total withholding</div>
                    <div class="value">$${totalWithholding.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s871m.card.treaty_saved">Saved by treaty</div>
                    <div class="value">$${treatySavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
