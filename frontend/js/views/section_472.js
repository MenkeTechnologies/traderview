// IRC § 472 — Last-In, First-Out (LIFO) Inventory Method Election.
// Election: identify ending inventory at COST of most recent acquisitions.
// During inflation: HIGHER COGS + LOWER taxable income vs FIFO.
// Book conformity rule (§ 472(c)): financial reporting MUST also use LIFO.
// Dollar-Value LIFO: aggregates similar items into pools by index.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    beginning_inventory: 0,
    ending_inventory_fifo: 0,
    ending_inventory_lifo: 0,
    cogs_fifo: 0,
    cogs_lifo: 0,
    purchases: 0,
    price_increase_pct: 5,
    lifo_election_year: 2024,
    book_conformity: true,
    is_dollar_value_lifo: false,
    inventory_layers_count: 0,
    lifo_reserve: 0,
    elected_simplified_dollar_value: false,
    is_c_to_s_conversion: false,
    cumulative_lifo_recapture: 0,
    pool_count: 0,
};

export async function renderSection472(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s472.h1.title">// § 472 LIFO INVENTORY METHOD</span></h1>
        <p class="muted small" data-i18n="view.s472.hint.intro">
            <strong>LIFO election:</strong> identify ending inventory at COST of <strong>most recent acquisitions</strong>.
            During inflation: HIGHER COGS + LOWER taxable income vs FIFO. <strong>Book conformity rule
            (§ 472(c)):</strong> financial reporting MUST also use LIFO. <strong>Dollar-Value LIFO:</strong>
            aggregates similar items into pools by index. <strong>LIFO recapture (§ 1363(d)):</strong> 4-year
            recapture on C-corp to S-corp conversion. <strong>§ 472(e):</strong> termination of LIFO method.
            <strong>Form 970</strong> filed to elect. IFRS does NOT allow LIFO (global comparison).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s472.h2.inputs">Inputs</h2>
            <form id="s472-form" class="inline-form">
                <label><span data-i18n="view.s472.label.beg">Beginning inventory ($)</span>
                    <input type="number" step="0.01" name="beginning_inventory" value="${state.beginning_inventory}"></label>
                <label><span data-i18n="view.s472.label.end_fifo">Ending inventory FIFO ($)</span>
                    <input type="number" step="0.01" name="ending_inventory_fifo" value="${state.ending_inventory_fifo}"></label>
                <label><span data-i18n="view.s472.label.end_lifo">Ending inventory LIFO ($)</span>
                    <input type="number" step="0.01" name="ending_inventory_lifo" value="${state.ending_inventory_lifo}"></label>
                <label><span data-i18n="view.s472.label.cogs_fifo">COGS FIFO ($)</span>
                    <input type="number" step="0.01" name="cogs_fifo" value="${state.cogs_fifo}"></label>
                <label><span data-i18n="view.s472.label.cogs_lifo">COGS LIFO ($)</span>
                    <input type="number" step="0.01" name="cogs_lifo" value="${state.cogs_lifo}"></label>
                <label><span data-i18n="view.s472.label.purchases">Purchases this period ($)</span>
                    <input type="number" step="0.01" name="purchases" value="${state.purchases}"></label>
                <label><span data-i18n="view.s472.label.price">Price increase %</span>
                    <input type="number" step="0.1" name="price_increase_pct" value="${state.price_increase_pct}"></label>
                <label><span data-i18n="view.s472.label.year">LIFO election year</span>
                    <input type="number" step="1" name="lifo_election_year" value="${state.lifo_election_year}"></label>
                <label><span data-i18n="view.s472.label.book">Book conformity met?</span>
                    <input type="checkbox" name="book_conformity" ${state.book_conformity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s472.label.dv">Dollar-Value LIFO?</span>
                    <input type="checkbox" name="is_dollar_value_lifo" ${state.is_dollar_value_lifo ? 'checked' : ''}></label>
                <label><span data-i18n="view.s472.label.layers">Inventory layers count</span>
                    <input type="number" step="1" name="inventory_layers_count" value="${state.inventory_layers_count}"></label>
                <label><span data-i18n="view.s472.label.reserve">LIFO reserve ($)</span>
                    <input type="number" step="0.01" name="lifo_reserve" value="${state.lifo_reserve}"></label>
                <label><span data-i18n="view.s472.label.simplified">Simplified Dollar-Value LIFO?</span>
                    <input type="checkbox" name="elected_simplified_dollar_value" ${state.elected_simplified_dollar_value ? 'checked' : ''}></label>
                <label><span data-i18n="view.s472.label.c_to_s">C-corp to S-corp conversion?</span>
                    <input type="checkbox" name="is_c_to_s_conversion" ${state.is_c_to_s_conversion ? 'checked' : ''}></label>
                <label><span data-i18n="view.s472.label.cum_recap">Cumulative LIFO recapture ($)</span>
                    <input type="number" step="0.01" name="cumulative_lifo_recapture" value="${state.cumulative_lifo_recapture}"></label>
                <label><span data-i18n="view.s472.label.pools">Pool count</span>
                    <input type="number" step="1" name="pool_count" value="${state.pool_count}"></label>
                <button class="primary" type="submit" data-i18n="view.s472.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s472-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s472.h2.mechanics">LIFO mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s472.mech.layers">"Layers": each year's purchases at year-end cost; oldest at bottom</li>
                <li data-i18n="view.s472.mech.cogs">COGS = most recent purchases (top of LIFO stack)</li>
                <li data-i18n="view.s472.mech.ending">Ending inventory: older layers preserved (potentially decades old)</li>
                <li data-i18n="view.s472.mech.specific_goods">Specific goods LIFO: track each item type separately</li>
                <li data-i18n="view.s472.mech.dollar_value">Dollar-Value LIFO: aggregate by FMV index (less complex tracking)</li>
                <li data-i18n="view.s472.mech.simplified_dollar">Simplified Dollar-Value LIFO: BLS / IPIC index allowed for smaller business</li>
                <li data-i18n="view.s472.mech.lifo_reserve">LIFO Reserve: difference between FIFO and LIFO inventory = deferred income</li>
                <li data-i18n="view.s472.mech.liquidation">LIFO liquidation: dipping into old layers releases low-cost inventory → income spike</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s472.h2.book_conformity">§ 472(c) book conformity rule</h2>
            <ul class="muted small">
                <li data-i18n="view.s472.bc.basic">REQUIREMENT: use LIFO for FINANCIAL reporting too (GAAP / IFRS)</li>
                <li data-i18n="view.s472.bc.cumulative_only">Disclosure of LIFO Reserve in financial statements: REQUIRED + ALLOWED</li>
                <li data-i18n="view.s472.bc.cost_method">"Cost method" — historical cost basis for LIFO</li>
                <li data-i18n="view.s472.bc.ifrs">IFRS conversion: IFRS PROHIBITS LIFO — forces LIFO termination if going IFRS</li>
                <li data-i18n="view.s472.bc.consequence">Violation: § 472(c) revokes LIFO election + retroactive FIFO</li>
                <li data-i18n="view.s472.bc.disclosures_required">"As-if FIFO" disclosure permitted (LIFO Reserve note)</li>
                <li data-i18n="view.s472.bc.parent_sub">Consolidated reporting: each entity's election counts separately</li>
                <li data-i18n="view.s472.bc.terminate">Termination: § 472(e) — recapture LIFO reserve over 3-4 years (Form 3115 method change)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s472.h2.lifo_recapture">§ 1363(d) C-to-S LIFO Recapture</h2>
            <ul class="muted small">
                <li data-i18n="view.s472.rec.basic">C-corp using LIFO converting to S-corp: 4-yr recapture of LIFO Reserve</li>
                <li data-i18n="view.s472.rec.formula">Recapture amount = LIFO Reserve = FIFO basis - LIFO basis</li>
                <li data-i18n="view.s472.rec.payments">25% recapture in C-corp final year + 25% × 3 yrs by S-corp</li>
                <li data-i18n="view.s472.rec.installments">Section 1378 installment provisions: pay over 4 yrs interest-free</li>
                <li data-i18n="view.s472.rec.aging_inventory">Common in long-established businesses with significant LIFO reserves</li>
                <li data-i18n="view.s472.rec.partial_election">Partial S-election (split between active + holding) may reduce trigger</li>
                <li data-i18n="view.s472.rec.recent_examples">Recent: oil + gas + heavy industry with old LIFO layers</li>
                <li data-i18n="view.s472.rec.estate_planning">Estate planning: consider LIFO recapture before S-election + heirs</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s472.h2.tradeoffs">LIFO vs FIFO trade-offs</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s472.th.aspect">Aspect</th>
                    <th data-i18n="view.s472.th.lifo">LIFO</th>
                    <th data-i18n="view.s472.th.fifo">FIFO</th>
                </tr></thead>
                <tbody>
                    <tr><td>Inflation environment</td><td>Higher COGS, lower income (tax saving)</td><td>Lower COGS, higher income</td></tr>
                    <tr><td>Deflation environment</td><td>Lower COGS, higher income</td><td>Higher COGS, lower income (tax saving)</td></tr>
                    <tr><td>Book conformity</td><td>Required GAAP</td><td>Not required</td></tr>
                    <tr><td>IFRS compatibility</td><td>NOT ALLOWED — bars IFRS adoption</td><td>Allowed</td></tr>
                    <tr><td>Liquidation risk</td><td>HIGH — tax bill on income spike</td><td>None</td></tr>
                    <tr><td>Earnings volatility</td><td>Lower (matched current prices)</td><td>Higher (mismatched cost vs revenue)</td></tr>
                    <tr><td>Complexity</td><td>Higher (layers / pools tracking)</td><td>Lower</td></tr>
                    <tr><td>Most common industries</td><td>Oil + gas, retail, automotive</td><td>Tech, services, perishables</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s472-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.beginning_inventory = Number(fd.get('beginning_inventory')) || 0;
        state.ending_inventory_fifo = Number(fd.get('ending_inventory_fifo')) || 0;
        state.ending_inventory_lifo = Number(fd.get('ending_inventory_lifo')) || 0;
        state.cogs_fifo = Number(fd.get('cogs_fifo')) || 0;
        state.cogs_lifo = Number(fd.get('cogs_lifo')) || 0;
        state.purchases = Number(fd.get('purchases')) || 0;
        state.price_increase_pct = Number(fd.get('price_increase_pct')) || 0;
        state.lifo_election_year = Number(fd.get('lifo_election_year')) || 0;
        state.book_conformity = !!fd.get('book_conformity');
        state.is_dollar_value_lifo = !!fd.get('is_dollar_value_lifo');
        state.inventory_layers_count = Number(fd.get('inventory_layers_count')) || 0;
        state.lifo_reserve = Number(fd.get('lifo_reserve')) || 0;
        state.elected_simplified_dollar_value = !!fd.get('elected_simplified_dollar_value');
        state.is_c_to_s_conversion = !!fd.get('is_c_to_s_conversion');
        state.cumulative_lifo_recapture = Number(fd.get('cumulative_lifo_recapture')) || 0;
        state.pool_count = Number(fd.get('pool_count')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s472-output');
    if (!el) return;
    const tax_savings = (state.cogs_lifo - state.cogs_fifo) * 0.21;
    const lifo_reserve = state.ending_inventory_fifo - state.ending_inventory_lifo;
    const annual_recapture = state.is_c_to_s_conversion ? state.cumulative_lifo_recapture / 4 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s472.h2.result">§ 472 LIFO computation</h2>
            <div class="cards">
                <div class="card ${state.book_conformity ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s472.card.conformity">Book conformity?</div>
                    <div class="value">${state.book_conformity ? esc(t('view.s472.status.yes')) : esc(t('view.s472.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s472.card.cogs_diff">COGS difference (LIFO - FIFO)</div>
                    <div class="value">$${(state.cogs_lifo - state.cogs_fifo).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s472.card.tax_savings">Current tax savings (21%)</div>
                    <div class="value">$${tax_savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s472.card.reserve">LIFO Reserve</div>
                    <div class="value">$${lifo_reserve.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s472.card.layers">Inventory layers</div>
                    <div class="value">${state.inventory_layers_count}</div>
                </div>
                <div class="card ${state.is_c_to_s_conversion ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s472.card.recapture">C-to-S annual recapture</div>
                    <div class="value">$${annual_recapture.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!state.book_conformity ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s472.no_conformity_note">
                    Book conformity rule VIOLATED — § 472(c) requires LIFO for financial reporting too.
                    IRS may revoke LIFO election retroactively, requiring FIFO recomputation + interest +
                    penalties on understated tax. Disclose LIFO Reserve in financial statement notes;
                    don't show "as-if FIFO" inventory on main balance sheet.
                </p>
            ` : ''}
        </div>
    `;
}
