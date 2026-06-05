// § 263A UNICAP — Uniform Capitalization Rules + Small Biz exemption.
// Producers + resellers must capitalize indirect costs into inventory + self-constructed.
// Small biz exemption: avg gross receipts ≤ $30M (2024, adjusted) over 3 prior years.
// 2017 TCJA raised from $10M. Form 3115 method change required to adopt or revoke.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SMALL_BIZ_GROSS_RECEIPTS_2024 = 30_000_000;

let state = {
    avg_gross_receipts: 0,
    business_activity: 'producer',
    direct_materials: 0,
    direct_labor: 0,
    indirect_storage: 0,
    indirect_purchasing: 0,
    indirect_handling: 0,
    indirect_admin: 0,
    indirect_distribution: 0,
    ending_inventory_pct: 0.30,
    marginal_rate: 0.32,
};

export async function renderSection263a(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s263a.h1.title">// § 263A UNICAP + SMALL BIZ EXEMPTION</span></h1>
        <p class="muted small" data-i18n="view.s263a.hint.intro">
            Producers + resellers must capitalize <strong>indirect costs into inventory</strong>
            and self-constructed assets — defers deduction until inventory sold or asset disposed.
            <strong>Small biz exemption:</strong> 3-yr avg gross receipts ≤ $30M (2024, indexed),
            no UNICAP required. TCJA raised threshold from $10M. Form 3115 method change required.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s263a.h2.exempt_test">Exemption test</h2>
            <form id="s263a-form" class="inline-form">
                <label><span data-i18n="view.s263a.label.avg_receipts">3-yr avg gross receipts ($)</span>
                    <input type="number" step="0.01" name="avg_gross_receipts" value="${state.avg_gross_receipts}"></label>
                <label><span data-i18n="view.s263a.label.activity">Business activity</span>
                    <select name="business_activity">
                        <option value="producer" ${state.business_activity === 'producer' ? 'selected' : ''}>Producer / Manufacturer</option>
                        <option value="reseller" ${state.business_activity === 'reseller' ? 'selected' : ''}>Reseller / Wholesale / Retail</option>
                        <option value="services" ${state.business_activity === 'services' ? 'selected' : ''}>Services only (UNICAP doesn't apply)</option>
                    </select>
                </label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s263a.label.direct_materials">Direct materials ($)</span>
                    <input type="number" step="0.01" name="direct_materials" value="${state.direct_materials}"></label>
                <label><span data-i18n="view.s263a.label.direct_labor">Direct labor ($)</span>
                    <input type="number" step="0.01" name="direct_labor" value="${state.direct_labor}"></label>
                <label><span data-i18n="view.s263a.label.indirect_storage">Indirect storage / warehouse ($)</span>
                    <input type="number" step="0.01" name="indirect_storage" value="${state.indirect_storage}"></label>
                <label><span data-i18n="view.s263a.label.indirect_purchasing">Indirect purchasing ($)</span>
                    <input type="number" step="0.01" name="indirect_purchasing" value="${state.indirect_purchasing}"></label>
                <label><span data-i18n="view.s263a.label.indirect_handling">Indirect handling ($)</span>
                    <input type="number" step="0.01" name="indirect_handling" value="${state.indirect_handling}"></label>
                <label><span data-i18n="view.s263a.label.indirect_admin">Indirect admin / overhead ($)</span>
                    <input type="number" step="0.01" name="indirect_admin" value="${state.indirect_admin}"></label>
                <label><span data-i18n="view.s263a.label.indirect_distribution">Indirect distribution / shipping ($)</span>
                    <input type="number" step="0.01" name="indirect_distribution" value="${state.indirect_distribution}"></label>
                <label><span data-i18n="view.s263a.label.ending_pct">Ending inventory %</span>
                    <input type="number" step="0.01" name="ending_inventory_pct" value="${state.ending_inventory_pct}"></label>
                <label><span data-i18n="view.s263a.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s263a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s263a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s263a.h2.unicap_categories">UNICAP cost categories (must capitalize)</h2>
            <ul class="muted small">
                <li data-i18n="view.s263a.cap.storage">Storage / warehouse costs (rent, depreciation, utilities)</li>
                <li data-i18n="view.s263a.cap.purchasing">Purchasing dept salaries + benefits</li>
                <li data-i18n="view.s263a.cap.handling">Material handling — receiving, inspection, picking</li>
                <li data-i18n="view.s263a.cap.admin">Production admin (supervisor wages, plant management)</li>
                <li data-i18n="view.s263a.cap.depreciation">Depreciation on production equipment</li>
                <li data-i18n="view.s263a.cap.indirect_labor">Indirect labor (maintenance, QC, foremen)</li>
                <li data-i18n="view.s263a.cap.interest">Capitalized interest on long-life self-construction</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s263a.h2.exclusions">Excluded from UNICAP</h2>
            <ul class="muted small">
                <li data-i18n="view.s263a.excl.rd">Research & experimentation (§ 174)</li>
                <li data-i18n="view.s263a.excl.marketing">Marketing + selling + distribution after production</li>
                <li data-i18n="view.s263a.excl.officer">Officer compensation (non-production)</li>
                <li data-i18n="view.s263a.excl.tax_dept">Tax department + general admin</li>
                <li data-i18n="view.s263a.excl.casualty">Casualty losses on inventory</li>
                <li data-i18n="view.s263a.excl.warranty">Post-production warranty service</li>
            </ul>
        </div>
    `;
    document.getElementById('s263a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.avg_gross_receipts = Number(fd.get('avg_gross_receipts')) || 0;
        state.business_activity = fd.get('business_activity');
        state.direct_materials = Number(fd.get('direct_materials')) || 0;
        state.direct_labor = Number(fd.get('direct_labor')) || 0;
        state.indirect_storage = Number(fd.get('indirect_storage')) || 0;
        state.indirect_purchasing = Number(fd.get('indirect_purchasing')) || 0;
        state.indirect_handling = Number(fd.get('indirect_handling')) || 0;
        state.indirect_admin = Number(fd.get('indirect_admin')) || 0;
        state.indirect_distribution = Number(fd.get('indirect_distribution')) || 0;
        state.ending_inventory_pct = Number(fd.get('ending_inventory_pct')) || 0.30;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s263a-output');
    if (!el) return;
    const exempt = state.avg_gross_receipts <= SMALL_BIZ_GROSS_RECEIPTS_2024 || state.business_activity === 'services';
    const indirect = state.indirect_storage + state.indirect_purchasing + state.indirect_handling + state.indirect_admin;
    const directCosts = state.direct_materials + state.direct_labor;
    const additionalToInventory = exempt ? 0 : indirect * state.ending_inventory_pct;
    const taxDeferred = additionalToInventory;
    const cashflowDelta = taxDeferred * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s263a.h2.result">UNICAP impact</h2>
            <div class="cards">
                <div class="card ${exempt ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s263a.card.exempt">Small biz exempt?</div>
                    <div class="value">${exempt ? esc(t('view.s263a.status.yes')) : esc(t('view.s263a.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s263a.card.threshold">2024 threshold</div>
                    <div class="value">$${SMALL_BIZ_GROSS_RECEIPTS_2024.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s263a.card.direct">Direct costs</div>
                    <div class="value">$${directCosts.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s263a.card.indirect">Indirect costs</div>
                    <div class="value">$${indirect.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${additionalToInventory > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s263a.card.additional">Additional capitalized to inv</div>
                    <div class="value">$${additionalToInventory.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${cashflowDelta > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s263a.card.cashflow">Year cashflow impact (tax deferral)</div>
                    <div class="value">$${cashflowDelta.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${exempt ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s263a.note.exempt_advice">
                    Small biz exemption applies: expense indirect costs as incurred + simpler
                    inventory accounting allowed (cash basis OR no inventory method per § 471(c)).
                </p>
            ` : `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s263a.note.required">
                    UNICAP required. Use Simplified Production Method (SPM) or Simplified Resale
                    Method (SRM) absorption ratios. Form 3115 to change methods.
                </p>
            `}
        </div>
    `;
}
