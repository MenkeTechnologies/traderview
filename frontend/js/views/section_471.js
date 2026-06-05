// IRC § 471 — Inventory Methods.
// Required when production / purchase + sale of merchandise is income-producing factor.
// Small business exemption: ≤ $30M avg gross receipts may use cash method + treat inventory as non-incidental supplies.
// Cost-flow methods: FIFO (default), LIFO (§ 472 election), Specific ID, Weighted Average.
// LIFO requires conformity with book reporting + LIFO recapture if S-corp converts from C.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SMALL_BIZ_EXEMPTION_2024 = 30_000_000;

let state = {
    cost_flow_method: 'fifo',
    avg_gross_receipts_3yr: 0,
    beginning_inventory: 0,
    purchases: 0,
    ending_inventory_fifo: 0,
    ending_inventory_lifo: 0,
    rising_or_falling_prices: 'rising',
    is_lifo_election: false,
    is_book_conforming: false,
    is_lcm_writedown: false,
    marginal_rate: 0.21,
};

export async function renderSection471(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s471.h1.title">// § 471 INVENTORY METHODS</span></h1>
        <p class="muted small" data-i18n="view.s471.hint.intro">
            Required when production / purchase + sale of merchandise = income factor.
            <strong>Small business exemption: ≤ $30M avg gross receipts</strong> may use cash
            method + treat inventory as non-incidental supplies. Cost-flow methods: <strong>FIFO
            (default), LIFO (§ 472 election), Specific ID, Weighted Average</strong>. LIFO requires
            <strong>book conformity</strong> + LIFO recapture if S-corp converts from C.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s471.h2.inputs">Inputs</h2>
            <form id="s471-form" class="inline-form">
                <label><span data-i18n="view.s471.label.method">Cost flow method</span>
                    <select name="cost_flow_method">
                        <option value="fifo" ${state.cost_flow_method === 'fifo' ? 'selected' : ''}>FIFO (First In, First Out)</option>
                        <option value="lifo" ${state.cost_flow_method === 'lifo' ? 'selected' : ''}>LIFO (Last In, First Out)</option>
                        <option value="specific_id" ${state.cost_flow_method === 'specific_id' ? 'selected' : ''}>Specific Identification</option>
                        <option value="weighted_average" ${state.cost_flow_method === 'weighted_average' ? 'selected' : ''}>Weighted Average</option>
                        <option value="retail" ${state.cost_flow_method === 'retail' ? 'selected' : ''}>Retail Method</option>
                        <option value="non_incidental" ${state.cost_flow_method === 'non_incidental' ? 'selected' : ''}>Non-incidental supplies (small biz)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s471.label.gross_3yr">Avg gross receipts 3-yr ($)</span>
                    <input type="number" step="0.01" name="avg_gross_receipts_3yr" value="${state.avg_gross_receipts_3yr}"></label>
                <label><span data-i18n="view.s471.label.beg_inv">Beginning inventory ($)</span>
                    <input type="number" step="0.01" name="beginning_inventory" value="${state.beginning_inventory}"></label>
                <label><span data-i18n="view.s471.label.purchases">Purchases during year ($)</span>
                    <input type="number" step="0.01" name="purchases" value="${state.purchases}"></label>
                <label><span data-i18n="view.s471.label.end_fifo">Ending inventory FIFO ($)</span>
                    <input type="number" step="0.01" name="ending_inventory_fifo" value="${state.ending_inventory_fifo}"></label>
                <label><span data-i18n="view.s471.label.end_lifo">Ending inventory LIFO ($)</span>
                    <input type="number" step="0.01" name="ending_inventory_lifo" value="${state.ending_inventory_lifo}"></label>
                <label><span data-i18n="view.s471.label.price_direction">Price direction</span>
                    <select name="rising_or_falling_prices">
                        <option value="rising" ${state.rising_or_falling_prices === 'rising' ? 'selected' : ''}>Rising prices</option>
                        <option value="falling" ${state.rising_or_falling_prices === 'falling' ? 'selected' : ''}>Falling prices</option>
                        <option value="stable" ${state.rising_or_falling_prices === 'stable' ? 'selected' : ''}>Stable prices</option>
                    </select>
                </label>
                <label><span data-i18n="view.s471.label.election">LIFO election (§ 472)?</span>
                    <input type="checkbox" name="is_lifo_election" ${state.is_lifo_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s471.label.book_conform">Book conformity?</span>
                    <input type="checkbox" name="is_book_conforming" ${state.is_book_conforming ? 'checked' : ''}></label>
                <label><span data-i18n="view.s471.label.lcm">LCM writedown taken?</span>
                    <input type="checkbox" name="is_lcm_writedown" ${state.is_lcm_writedown ? 'checked' : ''}></label>
                <label><span data-i18n="view.s471.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s471.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s471-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s471.h2.lifo_pros_cons">LIFO pros + cons</h2>
            <ul class="muted small">
                <li data-i18n="view.s471.lifo.pros">Pros: lower COGS in rising prices = lower current taxes; better cash flow</li>
                <li data-i18n="view.s471.lifo.cons">Cons: lower reported earnings; book conformity rule (§ 472(c)); complex layers</li>
                <li data-i18n="view.s471.lifo.recapture">§ 1363(d) LIFO recapture: 4-year recapture on C→S conversion</li>
                <li data-i18n="view.s471.lifo.repeal">Often repealed when prices flat / falling (LIFO liquidation triggers)</li>
                <li data-i18n="view.s471.lifo.dollar_value">Dollar-value LIFO: aggregates by index pool</li>
                <li data-i18n="view.s471.lifo.unicap">§ 263A UNICAP rules add complexity for LIFO</li>
                <li data-i18n="view.s471.lifo.global_repeal">Global IFRS doesn't allow LIFO (some companies repeal US LIFO too)</li>
                <li data-i18n="view.s471.lifo.simplified">Simplified LIFO available for retailers (BLS index)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s471.h2.small_biz_exemption">§ 471(c) Small Business Inventory Exemption</h2>
            <ul class="muted small">
                <li data-i18n="view.s471.sbe.eligible">Avg gross receipts 3-yr ≤ $30M (2024, inflation-indexed)</li>
                <li data-i18n="view.s471.sbe.cash_method">Can use cash method even with inventory</li>
                <li data-i18n="view.s471.sbe.non_incidental">Inventory treated as non-incidental materials and supplies</li>
                <li data-i18n="view.s471.sbe.deduct_when_paid">Deduct cost in year PAID + provided for sale/use</li>
                <li data-i18n="view.s471.sbe.no_lcm">No § 263A UNICAP required</li>
                <li data-i18n="view.s471.sbe.no_unicap">No write-down or LCM required either</li>
                <li data-i18n="view.s471.sbe.book_conformity_optional">Book conformity not required</li>
                <li data-i18n="view.s471.sbe.elect">Election made on Form 3115 with § 481(a) adjustment</li>
            </ul>
        </div>
    `;
    document.getElementById('s471-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.cost_flow_method = fd.get('cost_flow_method');
        state.avg_gross_receipts_3yr = Number(fd.get('avg_gross_receipts_3yr')) || 0;
        state.beginning_inventory = Number(fd.get('beginning_inventory')) || 0;
        state.purchases = Number(fd.get('purchases')) || 0;
        state.ending_inventory_fifo = Number(fd.get('ending_inventory_fifo')) || 0;
        state.ending_inventory_lifo = Number(fd.get('ending_inventory_lifo')) || 0;
        state.rising_or_falling_prices = fd.get('rising_or_falling_prices');
        state.is_lifo_election = !!fd.get('is_lifo_election');
        state.is_book_conforming = !!fd.get('is_book_conforming');
        state.is_lcm_writedown = !!fd.get('is_lcm_writedown');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.21;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s471-output');
    if (!el) return;
    const totalAvailable = state.beginning_inventory + state.purchases;
    const cogs_fifo = totalAvailable - state.ending_inventory_fifo;
    const cogs_lifo = totalAvailable - state.ending_inventory_lifo;
    const cogs_difference = cogs_lifo - cogs_fifo;
    const taxSavingsFromLifo = Math.max(0, cogs_difference) * state.marginal_rate;
    const smallBizEligible = state.avg_gross_receipts_3yr <= SMALL_BIZ_EXEMPTION_2024;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s471.h2.result">Inventory + COGS analysis</h2>
            <div class="cards">
                <div class="card ${smallBizEligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s471.card.small_biz">Small biz exemption</div>
                    <div class="value">${smallBizEligible ? esc(t('view.s471.status.yes')) : esc(t('view.s471.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s471.card.available">Available for sale</div>
                    <div class="value">$${totalAvailable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s471.card.cogs_fifo">COGS (FIFO)</div>
                    <div class="value">$${cogs_fifo.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s471.card.cogs_lifo">COGS (LIFO)</div>
                    <div class="value">$${cogs_lifo.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${cogs_difference > 0 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s471.card.difference">LIFO higher COGS by</div>
                    <div class="value">$${cogs_difference.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s471.card.tax_savings">LIFO tax savings vs FIFO</div>
                    <div class="value">$${taxSavingsFromLifo.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
