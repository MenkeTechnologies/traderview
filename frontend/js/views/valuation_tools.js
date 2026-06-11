// Valuation tools — Reverse DCF, Dividend Discount, Earnings Power
// Value, and the options Wheel calculator behind one tabbed view. Each
// tool is a server-side pure-compute POST; this view is form + result.

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const TOOLS = {
    'reverse-dcf': {
        label: 'Reverse DCF',
        call: (b) => api.calcReverseDcf(b),
        fields: [
            { key: 'fcf_usd', label: 'Free cash flow (TTM, $)', def: 100_000_000_000 },
            { key: 'growth_years', label: 'Growth years', def: 5, int: true },
            { key: 'terminal_growth_pct', label: 'Terminal growth (%)', def: 2.5 },
            { key: 'discount_rate_pct', label: 'Discount rate (%)', def: 10 },
            { key: 'net_debt_usd', label: 'Net debt ($)', def: 0 },
            { key: 'shares_outstanding', label: 'Shares outstanding', def: 15_000_000_000 },
            { key: 'current_price', label: 'Current price ($)', def: 200 },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Implied growth</div>
                    <div class="value">${r.implied_growth_pct.toFixed(2)}%/yr</div>
                    <div class="small muted">market-priced stage-1 FCF growth</div></div>
                <div class="card"><div class="label">Check value</div>
                    <div class="value">$${r.intrinsic_at_implied.toFixed(2)}</div></div>
            </div>
            <p class="muted small">If you believe the business will grow FASTER than the implied
            rate, the stock is cheap by your own assumptions — and vice versa.</p>`,
    },
    ddm: {
        label: 'Dividend Discount',
        call: (b) => api.calcDdm(b),
        fields: [
            { key: 'annual_dividend', label: 'Forward dividend / share ($)', def: 4 },
            { key: 'growth_pct', label: 'Stage-1 growth (%/yr)', def: 6 },
            { key: 'growth_years', label: 'Stage-1 years (0 = Gordon)', def: 5, int: true },
            { key: 'terminal_growth_pct', label: 'Terminal growth (%)', def: 2.5 },
            { key: 'required_return_pct', label: 'Required return (%)', def: 9 },
            { key: 'current_price', label: 'Current price ($, optional)', def: '', optional: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Fair value (${esc(r.model)})</div>
                    <div class="value">$${r.fair_value.toFixed(2)}</div>
                    ${r.upside_pct != null ? `<div class="small ${r.upside_pct >= 0 ? 'pos' : 'neg'}">${(r.upside_pct >= 0 ? '+' : '') + r.upside_pct.toFixed(1)}% vs price</div>` : ''}
                </div>
            </div>`,
    },
    epv: {
        label: 'Earnings Power',
        call: (b) => api.calcEpv(b),
        fields: [
            { key: 'normalized_ebit_usd', label: 'Normalized EBIT ($)', def: 10_000_000_000 },
            { key: 'tax_rate_pct', label: 'Tax rate (%)', def: 21 },
            { key: 'wacc_pct', label: 'WACC (%)', def: 9 },
            { key: 'net_cash_adjustment_usd', label: 'Net cash adj. ($)', def: 0 },
            { key: 'shares_outstanding', label: 'Shares outstanding', def: 1_000_000_000 },
            { key: 'current_price', label: 'Current price ($, optional)', def: '', optional: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">EPV / share</div>
                    <div class="value">$${r.epv_per_share.toFixed(2)}</div>
                    ${r.upside_pct != null ? `<div class="small ${r.upside_pct >= 0 ? 'pos' : 'neg'}">${(r.upside_pct >= 0 ? '+' : '') + r.upside_pct.toFixed(1)}% vs price</div>` : ''}
                </div>
                <div class="card"><div class="label">NOPAT</div><div class="value">$${(r.nopat_usd / 1e9).toFixed(2)}B</div></div>
                <div class="card"><div class="label">EPV (operations)</div><div class="value">$${(r.epv_operations_usd / 1e9).toFixed(2)}B</div></div>
            </div>
            <p class="muted small">Greenwald EPV: value at ZERO growth. Price above EPV means
            the market is paying for growth; below means even no-growth earnings cover it.</p>`,
    },
    wheel: {
        label: 'Wheel Calculator',
        call: (b) => api.calcWheel(b),
        fields: [
            { key: 'stock_price', label: 'Stock price ($)', def: 100 },
            { key: 'put_strike', label: 'CSP strike ($)', def: 95 },
            { key: 'put_premium', label: 'CSP premium ($)', def: 2 },
            { key: 'put_dte', label: 'CSP days to expiry', def: 30, int: true },
            { key: 'call_strike', label: 'CC strike ($)', def: 100 },
            { key: 'call_premium', label: 'CC premium ($)', def: 2.5 },
            { key: 'call_dte', label: 'CC days to expiry', def: 30, int: true },
        ],
        render: (r) => `
            <div class="cards">
                <div class="card"><div class="label">Put-side return</div>
                    <div class="value">${r.put_side_return_pct.toFixed(2)}%</div>
                    <div class="small pos">${r.put_side_annualized_pct.toFixed(1)}% annualized</div></div>
                <div class="card"><div class="label">Full-cycle return</div>
                    <div class="value">${r.full_cycle_return_pct.toFixed(2)}%</div>
                    <div class="small pos">${r.full_cycle_annualized_pct.toFixed(1)}% ann. / ${r.full_cycle_days}d</div></div>
                <div class="card"><div class="label">Assigned basis</div>
                    <div class="value">$${r.assigned_cost_basis.toFixed(2)}</div>
                    <div class="small muted">${r.breakeven_drop_pct.toFixed(1)}% cushion</div></div>
            </div>`,
    },
};

export async function renderValuationTools(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.valuation_tools.h1" class="view-title">// VALUATION TOOLS</h1>
        <div class="gs-filter-row vt-tabs">
            ${Object.entries(TOOLS).map(([k, v]) => `
                <button class="btn btn-secondary gs-filter vt-tab" data-key="${k}">${esc(v.label)}</button>
            `).join('')}
        </div>
        <div class="chart-panel"><div id="vt-body"></div></div>
        <div class="chart-panel"><div id="vt-result" class="muted" data-i18n="view.valuation_tools.hint">Pick a tool, fill the form, hit Compute.</div></div>
    `;
    try { applyUiI18n(mount); } catch (_) {}

    const body = mount.querySelector('#vt-body');
    const out = mount.querySelector('#vt-result');

    const show = (key) => {
        const tool = TOOLS[key];
        mount.querySelectorAll('.vt-tab').forEach(b =>
            b.classList.toggle('active', b.dataset.key === key));
        body.innerHTML = `
            <form class="dcf-form" data-tool="${key}">
                ${tool.fields.map(f => `
                    <label class="dcf-field">
                        <span class="dcf-label">${esc(f.label)}</span>
                        <input name="${f.key}" type="number" step="any" value="${f.def}">
                    </label>`).join('')}
                <button type="submit" class="primary">Compute</button>
            </form>`;
        body.querySelector('form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const fd = new FormData(e.target);
            const payload = {};
            for (const f of tool.fields) {
                const raw = fd.get(f.key);
                if (f.optional && raw === '') { payload[f.key] = null; continue; }
                payload[f.key] = f.int ? (parseInt(raw, 10) || 0) : (Number(raw) || 0);
            }
            out.textContent = '…';
            try {
                const r = await tool.call(payload);
                if (!viewIsCurrent(tok)) return;
                out.innerHTML = tool.render(r);
            } catch (err) {
                out.innerHTML = `<span class="neg">${esc(err.message || String(err))}</span>`;
            }
        });
    };
    mount.querySelectorAll('.vt-tab').forEach(b =>
        b.addEventListener('click', () => show(b.dataset.key)));
    show('reverse-dcf');
}
