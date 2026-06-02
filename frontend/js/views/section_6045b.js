// IRC § 6045B — Broker Cost Basis Reporting + Corporate Actions.
// Issuers of securities must file Form 8937 within 45 days of any "specified action affecting basis"
// (spinoffs, mergers, splits, mandatory exchanges). Public on issuer website.
// Brokers (§ 6045) report adjusted basis on 1099-B (covered after 2011 for stock, 2012 for mutual funds).
// Penalty: $310 per failure (2024), $3.78M annual cap.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PENALTY_PER_FAILURE_2024 = 310;
const PENALTY_INTENTIONAL = 630;
const ANNUAL_CAP = 3_780_000;

let state = {
    is_issuer: true,
    is_broker: false,
    securities_affected: 0,
    days_late: 0,
    intentional_disregard: false,
    is_specified_action: true,
    action_kind: 'spinoff',
    notice_provided_to_holders: true,
    posted_on_website: true,
};

export async function renderSection6045b(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6045b.h1.title">// § 6045B FORM 8937 COST BASIS</span></h1>
        <p class="muted small" data-i18n="view.s6045b.hint.intro">
            <strong>Issuers must file Form 8937 within 45 days</strong> of any "specified action
            affecting basis" — spinoffs, mergers, splits, mandatory exchanges. Public on issuer
            website. <strong>Brokers (§ 6045) report adjusted basis on 1099-B</strong> (covered
            stocks post-2011, mutual funds post-2012). <strong>Penalty: $310 per failure (2024)</strong>;
            $630 if intentional. Annual cap: $3.78M. § 6045B-1(d) safe harbor: posting on website
            with 30-day notice to holders.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045b.h2.inputs">Inputs</h2>
            <form id="s6045b-form" class="inline-form">
                <label><span data-i18n="view.s6045b.label.is_issuer">Issuer?</span>
                    <input type="checkbox" name="is_issuer" ${state.is_issuer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045b.label.is_broker">Broker (separate § 6045)?</span>
                    <input type="checkbox" name="is_broker" ${state.is_broker ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045b.label.count">Securities / holders affected</span>
                    <input type="number" step="1000" name="securities_affected" value="${state.securities_affected}"></label>
                <label><span data-i18n="view.s6045b.label.days_late">Days late</span>
                    <input type="number" step="1" name="days_late" value="${state.days_late}"></label>
                <label><span data-i18n="view.s6045b.label.intentional">Intentional disregard?</span>
                    <input type="checkbox" name="intentional_disregard" ${state.intentional_disregard ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045b.label.is_specified">Specified action affecting basis?</span>
                    <input type="checkbox" name="is_specified_action" ${state.is_specified_action ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045b.label.action_kind">Action kind</span>
                    <select name="action_kind">
                        <option value="spinoff" ${state.action_kind === 'spinoff' ? 'selected' : ''}>Spin-off (§ 355)</option>
                        <option value="merger" ${state.action_kind === 'merger' ? 'selected' : ''}>Merger / reorg (§ 368)</option>
                        <option value="stock_split" ${state.action_kind === 'stock_split' ? 'selected' : ''}>Stock split / dividend</option>
                        <option value="exchange" ${state.action_kind === 'exchange' ? 'selected' : ''}>Mandatory share exchange</option>
                        <option value="redemption" ${state.action_kind === 'redemption' ? 'selected' : ''}>Cash + stock partial redemption</option>
                        <option value="rights" ${state.action_kind === 'rights' ? 'selected' : ''}>Rights / warrant distribution</option>
                        <option value="contingent" ${state.action_kind === 'contingent' ? 'selected' : ''}>Contingent payment closing</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6045b.label.notice">Notice provided to holders within 15 days?</span>
                    <input type="checkbox" name="notice_provided_to_holders" ${state.notice_provided_to_holders ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6045b.label.posted">Posted on issuer website?</span>
                    <input type="checkbox" name="posted_on_website" ${state.posted_on_website ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6045b.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6045b-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045b.h2.covered_securities">Covered securities (post-2011)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6045b.cov.stock">Stock acquired after 1/1/2011 (corporate)</li>
                <li data-i18n="view.s6045b.cov.mutual_fund">Mutual fund + DRIP shares acquired after 1/1/2012</li>
                <li data-i18n="view.s6045b.cov.dividends">Debt securities acquired after 1/1/2014 + complex debt after 1/1/2016</li>
                <li data-i18n="view.s6045b.cov.options">Options (most types) after 1/1/2014</li>
                <li data-i18n="view.s6045b.cov.warrants">Warrants + rights after 1/1/2014</li>
                <li data-i18n="view.s6045b.cov.crypto_future">Crypto / digital assets: post-2025 reporting effective per IRA + IRS rules</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045b.h2.reportable_actions">Reportable corporate actions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6045b.act.stock_split">Stock splits, reverse splits, stock dividends &gt; 5%</li>
                <li data-i18n="view.s6045b.act.spinoff">Tax-free spinoffs (§ 355) + taxable spin-offs</li>
                <li data-i18n="view.s6045b.act.reorg">Mergers + reorganizations (§ 368)</li>
                <li data-i18n="view.s6045b.act.recap">Recapitalizations + reclassifications</li>
                <li data-i18n="view.s6045b.act.liquidations">§ 332 corporate liquidations</li>
                <li data-i18n="view.s6045b.act.bankrupt">Bankruptcy reorganizations</li>
                <li data-i18n="view.s6045b.act.partial">Partial liquidations (§ 302(b)(4))</li>
                <li data-i18n="view.s6045b.act.rights">Rights offerings + warrant distributions</li>
                <li data-i18n="view.s6045b.act.cash_in_lieu">Cash in lieu of fractional shares</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6045b.h2.broker_reporting">Broker (§ 6045) reporting flow</h2>
            <ul class="muted small">
                <li data-i18n="view.s6045b.broker.cov_uncov">"Covered" vs "non-covered" securities — only covered have basis reported</li>
                <li data-i18n="view.s6045b.broker.transfer">Transferring broker must send Form 1099-B + transfer statement within 15 days</li>
                <li data-i18n="view.s6045b.broker.elect_default">Default cost method: FIFO; can elect HIFO / specific ID before sale</li>
                <li data-i18n="view.s6045b.broker.wash_sale">Brokers must report wash-sale adjustments to basis</li>
                <li data-i18n="view.s6045b.broker.short_sale">Short sale basis: reporting begins year position closes</li>
                <li data-i18n="view.s6045b.broker.constructive">§ 1259 constructive sale: separate broker reporting category</li>
            </ul>
        </div>
    `;
    document.getElementById('s6045b-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_issuer = !!fd.get('is_issuer');
        state.is_broker = !!fd.get('is_broker');
        state.securities_affected = Number(fd.get('securities_affected')) || 0;
        state.days_late = Number(fd.get('days_late')) || 0;
        state.intentional_disregard = !!fd.get('intentional_disregard');
        state.is_specified_action = !!fd.get('is_specified_action');
        state.action_kind = fd.get('action_kind');
        state.notice_provided_to_holders = !!fd.get('notice_provided_to_holders');
        state.posted_on_website = !!fd.get('posted_on_website');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6045b-output');
    if (!el) return;
    const ratePerFailure = state.intentional_disregard ? PENALTY_INTENTIONAL : PENALTY_PER_FAILURE_2024;
    const grossPenalty = state.securities_affected * ratePerFailure * (state.days_late > 0 ? 1 : 0);
    const cappedPenalty = state.intentional_disregard ? grossPenalty : Math.min(grossPenalty, ANNUAL_CAP);
    const safeHarborOk = state.posted_on_website && state.notice_provided_to_holders;
    const finalPenalty = safeHarborOk && !state.intentional_disregard ? 0 : cappedPenalty;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6045b.h2.result">Compliance exposure</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6045b.card.per_failure">Rate / failure</div>
                    <div class="value">$${ratePerFailure.toLocaleString()}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6045b.card.gross">Gross penalty</div>
                    <div class="value">$${grossPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6045b.card.cap">Annual cap</div>
                    <div class="value">$${ANNUAL_CAP.toLocaleString()}</div>
                </div>
                <div class="card ${safeHarborOk ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6045b.card.safe_harbor">§ 6045B-1(d) safe harbor</div>
                    <div class="value">${safeHarborOk ? esc(t('view.s6045b.status.yes')) : esc(t('view.s6045b.status.no'))}</div>
                </div>
                <div class="card ${finalPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6045b.card.final">Final penalty</div>
                    <div class="value">$${finalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
