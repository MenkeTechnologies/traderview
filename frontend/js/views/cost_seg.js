// Cost Segregation Study Tracker — accelerated real-estate depreciation.
// Re-classify building components from 27.5/39 yr to 5/7/15 yr classes.
// Cost-seg study typically reclassifies 20-40% of basis. Bonus depreciation
// (60% 2024, 40% 2025, 20% 2026) applies to reclassified components.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-costseg-v1';
const BONUS_BY_YEAR = {
    2022: 1.00, 2023: 0.80, 2024: 0.60, 2025: 0.40, 2026: 0.20, 2027: 0.00,
};

const TYPICAL_RECLASS = {
    residential: { '5_year': 0.15, '15_year': 0.10, '27_5_year': 0.75 },
    commercial:  { '5_year': 0.20, '15_year': 0.12, '39_year': 0.68 },
    hotel:       { '5_year': 0.30, '15_year': 0.15, '39_year': 0.55 },
    industrial:  { '5_year': 0.18, '15_year': 0.13, '39_year': 0.69 },
};

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    properties: load(),
    marginal_rate: 0.35,
    study_cost: 4_500,
};

export async function renderCostSeg(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.costseg.h1.title">// COST SEGREGATION</span></h1>
        <p class="muted small" data-i18n="view.costseg.hint.intro">
            Engineering study reclassifies building components from 27.5/39 yr to
            5/7/15 yr classes — typically 20-40% of basis. Combined with bonus
            depreciation (60% in 2024), front-loads decades of depreciation into year-1.
            Study cost: $3-15k. Catch-up via Form 3115 (no amended returns needed).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.costseg.h2.add">Add property</h2>
            <form id="cs-form" class="inline-form">
                <label><span data-i18n="view.costseg.label.address">Property / address</span>
                    <input type="text" name="address" placeholder="123 Main St rental" required></label>
                <label><span data-i18n="view.costseg.label.property_type">Type</span>
                    <select name="property_type">
                        <option value="residential">Residential rental</option>
                        <option value="commercial">Commercial</option>
                        <option value="hotel">Hotel / short-term rental</option>
                        <option value="industrial">Industrial</option>
                    </select>
                </label>
                <label><span data-i18n="view.costseg.label.purchase_price">Purchase price ($)</span>
                    <input type="number" step="0.01" name="purchase_price" required></label>
                <label><span data-i18n="view.costseg.label.land_value">Land value (excluded) ($)</span>
                    <input type="number" step="0.01" name="land_value" required></label>
                <label><span data-i18n="view.costseg.label.placed_year">Year placed in service</span>
                    <input type="number" step="1" name="placed_year" value="${new Date().getFullYear()}" required></label>
                <button class="primary" type="submit" data-i18n="view.costseg.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.costseg.h2.context">Tax context</h2>
            <form id="cs-tax" class="inline-form">
                <label><span data-i18n="view.costseg.label.marginal_rate">Marginal federal+state %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.costseg.label.study_cost">Study cost ($)</span>
                    <input type="number" step="0.01" name="study_cost" value="${state.study_cost}"></label>
                <button class="primary" type="submit" data-i18n="view.costseg.btn.update">Update</button>
            </form>
        </div>
        <div id="cs-summary"></div>
        <div id="cs-table" class="chart-panel"></div>
    `;
    document.getElementById('cs-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const p = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            address: fd.get('address'),
            property_type: fd.get('property_type'),
            purchase_price: Number(fd.get('purchase_price')),
            land_value: Number(fd.get('land_value')),
            placed_year: Number(fd.get('placed_year')),
        };
        state.properties.push(p);
        save(state.properties);
        e.target.reset();
        e.target.querySelector('[name="placed_year"]').value = new Date().getFullYear();
        showToast(t('view.costseg.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('cs-tax').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 35) / 100;
        state.study_cost = Number(fd.get('study_cost')) || 0;
        render();
    });
    render();
}

function analyzeProperty(p) {
    const depreciable = p.purchase_price - p.land_value;
    const reclass = TYPICAL_RECLASS[p.property_type] || TYPICAL_RECLASS.residential;
    const fiveYear = depreciable * (reclass['5_year'] || 0);
    const fifteenYear = depreciable * (reclass['15_year'] || 0);
    const longLife = depreciable - fiveYear - fifteenYear;
    const bonus = BONUS_BY_YEAR[p.placed_year] || 0;
    // Year-1 accelerated depreciation: bonus on 5+15-year property + first-year straight-line on long life
    const bonusDep = (fiveYear + fifteenYear) * bonus;
    const remaining5Yr = (fiveYear * (1 - bonus)) * 0.20;
    const remaining15Yr = (fifteenYear * (1 - bonus)) * 0.05;
    const longLifeYr1 = longLife / (p.property_type === 'residential' ? 27.5 : 39);
    const year1Total = bonusDep + remaining5Yr + remaining15Yr + longLifeYr1;
    // Without cost-seg: pure straight-line
    const withoutCostSeg = depreciable / (p.property_type === 'residential' ? 27.5 : 39);
    return {
        depreciable, fiveYear, fifteenYear, longLife,
        bonus, bonusDep, year1Total, withoutCostSeg,
        accelerated: year1Total - withoutCostSeg,
        tax_benefit: (year1Total - withoutCostSeg) * state.marginal_rate,
    };
}

function render() {
    const stats = state.properties.map(analyzeProperty);
    renderSummary(stats);
    renderTable(stats);
}

function renderSummary(stats) {
    const el = document.getElementById('cs-summary');
    if (!el) return;
    const totalAccelerated = stats.reduce((s, x) => s + x.accelerated, 0);
    const totalTaxBenefit = stats.reduce((s, x) => s + x.tax_benefit, 0);
    const netBenefit = totalTaxBenefit - state.study_cost;
    const roi = state.study_cost > 0 ? netBenefit / state.study_cost : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.costseg.h2.summary">Aggregate impact</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.costseg.card.properties">Properties</div>
                    <div class="value">${state.properties.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.costseg.card.accelerated">Year-1 accelerated depreciation</div>
                    <div class="value">$${totalAccelerated.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.costseg.card.tax_benefit">Year-1 tax benefit</div>
                    <div class="value">$${totalTaxBenefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.costseg.card.study_cost">Study cost</div>
                    <div class="value">$${state.study_cost.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.costseg.card.net_benefit">Net benefit</div>
                    <div class="value">$${netBenefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.costseg.card.roi">ROI on study cost</div>
                    <div class="value">${roi >= 0 ? (roi * 100).toFixed(0) + '%' : '—'}</div>
                </div>
            </div>
            <p class="muted small" style="margin-top:10px" data-i18n="view.costseg.note">
                Caveat: accelerated depreciation is RECAPTURE'd as ordinary income (25%) at sale.
                Best for: real-estate pros, STR with material participation, holders 10+ years,
                or those planning § 1031 exchange (recapture continues to defer).
            </p>
        </div>
    `;
}

function renderTable(stats) {
    const el = document.getElementById('cs-table');
    if (!el) return;
    if (!state.properties.length) {
        el.innerHTML = `<h2 data-i18n="view.costseg.h2.properties">Properties</h2>
            <p class="muted" data-i18n="view.costseg.empty">No properties tracked yet.</p>`;
        return;
    }
    el.innerHTML = `
        <h2 data-i18n="view.costseg.h2.properties">Properties</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.costseg.th.address">Property</th>
                <th data-i18n="view.costseg.th.type">Type</th>
                <th data-i18n="view.costseg.th.purchase">Purchase</th>
                <th data-i18n="view.costseg.th.depreciable">Depreciable</th>
                <th data-i18n="view.costseg.th.5yr">5-yr reclass</th>
                <th data-i18n="view.costseg.th.15yr">15-yr reclass</th>
                <th data-i18n="view.costseg.th.bonus">Bonus %</th>
                <th data-i18n="view.costseg.th.year1">Year-1 dep</th>
                <th data-i18n="view.costseg.th.accelerated">Accelerated</th>
                <th data-i18n="view.costseg.th.tax_benefit">Tax benefit</th>
                <th data-i18n="view.costseg.th.actions">Actions</th>
            </tr></thead>
            <tbody>${state.properties.map((p, i) => {
                const s = stats[i];
                return `<tr>
                    <td><strong>${esc(p.address)}</strong></td>
                    <td class="muted">${esc(p.property_type)}</td>
                    <td>$${p.purchase_price.toLocaleString()}</td>
                    <td>$${s.depreciable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${s.fiveYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${s.fifteenYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${(s.bonus * 100).toFixed(0)}%</td>
                    <td class="pos">$${s.year1Total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${s.accelerated.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${s.tax_benefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(p.id)}" data-i18n="view.costseg.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.properties = state.properties.filter(p => p.id !== btn.dataset.del);
            save(state.properties);
            render();
        });
    });
}
