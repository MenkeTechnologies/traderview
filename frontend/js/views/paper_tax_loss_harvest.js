// Paper-account tax-loss harvest scanner. Distinct from the existing
// trades-table tax_loss_harvest view (which works off executions and
// realized-loss tracking). This one scans current paper positions for
// unrealized losses ≥ threshold, suggests replacement ETFs, flags
// wash-sale risk from paper_orders, estimates tax savings at user's
// marginal rate.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderPaperTaxLossHarvest(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.paper_tax_loss_harvest.title">// PAPER TAX-LOSS HARVEST</span></h1>
        <p class="muted small" data-i18n-html="view.paper_tax_loss_harvest.intro">
            For each long paper position with an unrealized loss ≥ threshold,
            suggests <strong>harvesting</strong> (selling to realize the loss for
            tax deduction) plus a <strong>replacement candidate</strong> that
            maintains sector exposure without violating IRC §1091 wash-sale rules.
            Wash-sale risk is flagged when any buy of the same symbol exists in
            the trailing 30 days. <strong>Paper account simulation only</strong> —
            real broker integration uses the existing trades-table tax-loss-harvest view.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label>
                    <span class="muted small" data-i18n="view.paper_tax_loss_harvest.field.rate">Marginal rate %</span>
                    <input type="number" id="pth-rate" step="1" min="0" max="60" value="35" style="width:80px">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.paper_tax_loss_harvest.field.threshold">Min loss %</span>
                    <input type="number" id="pth-threshold" step="0.5" min="0" max="100" value="5" style="width:80px">
                </label>
                <button class="btn btn-sm primary" id="pth-run" data-shortcut="r" data-i18n="view.paper_tax_loss_harvest.btn.run">⚡ Scan</button>
                <span class="muted small" id="pth-meta"></span>
            </div>
            <div id="pth-summary"></div>
            <div id="pth-result"></div>
        </div>
    `;
    mount.querySelector('#pth-run').addEventListener('click', () => runScan(mount));
    await runScan(mount);
}

async function runScan(mount) {
    const summary = mount.querySelector('#pth-summary');
    const result = mount.querySelector('#pth-result');
    const rate = parseFloat(mount.querySelector('#pth-rate').value) || 35;
    const threshold = parseFloat(mount.querySelector('#pth-threshold').value) || 5;
    summary.innerHTML = `<p class="muted">${esc(t('view.paper_tax_loss_harvest.status.scanning'))}</p>`;
    result.innerHTML = '';
    try {
        const r = await api(`/paper-tax-loss-harvest/scan?marginal_rate_pct=${rate}&min_loss_threshold_pct=${threshold}`);
        const n = (r.candidates || []).length;
        summary.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px">
                <div><div class="muted small">${esc(t('view.paper_tax_loss_harvest.field.candidates'))}</div>
                    <strong>${n}</strong></div>
                <div><div class="muted small">${esc(t('view.paper_tax_loss_harvest.field.total_loss'))}</div>
                    <strong class="neg">$${r.total_loss_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.paper_tax_loss_harvest.field.total_savings'))}</div>
                    <strong class="pos">$${r.total_tax_savings_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.paper_tax_loss_harvest.field.rate_used'))}</div>
                    <strong>${r.marginal_rate_pct.toFixed(0)}%</strong></div>
            </div>
        `;
        if (!n) {
            result.innerHTML = `<p class="muted">${esc(t('view.paper_tax_loss_harvest.empty.no_candidates'))}</p>`;
            return;
        }
        result.innerHTML = `
            <table class="trades" style="margin-top:1em">
                <thead><tr>
                    <th data-i18n="view.paper_tax_loss_harvest.th.symbol">Symbol</th>
                    <th data-i18n="view.paper_tax_loss_harvest.th.qty">Qty</th>
                    <th data-i18n="view.paper_tax_loss_harvest.th.cost">Cost/sh</th>
                    <th data-i18n="view.paper_tax_loss_harvest.th.price">Price</th>
                    <th data-i18n="view.paper_tax_loss_harvest.th.loss_pct">Loss %</th>
                    <th data-i18n="view.paper_tax_loss_harvest.th.loss_usd">Loss $</th>
                    <th data-i18n="view.paper_tax_loss_harvest.th.savings">Tax Savings</th>
                    <th data-i18n="view.paper_tax_loss_harvest.th.replacement">Replacement</th>
                    <th data-i18n="view.paper_tax_loss_harvest.th.wash_sale">Wash Sale?</th>
                </tr></thead>
                <tbody>${r.candidates.map(c => `
                    <tr>
                        <td><strong>${esc(c.symbol)}</strong></td>
                        <td>${c.qty.toFixed(0)}</td>
                        <td>$${c.cost_basis_per_share.toFixed(2)}</td>
                        <td>$${c.current_price.toFixed(2)}</td>
                        <td class="neg">${c.unrealized_loss_pct.toFixed(2)}%</td>
                        <td class="neg">$${c.unrealized_loss_usd.toFixed(2)}</td>
                        <td class="pos">$${c.estimated_tax_savings_usd.toFixed(2)}</td>
                        <td>${c.replacement_symbol ? `<strong>${esc(c.replacement_symbol)}</strong> <span class="muted small">${esc(c.replacement_rationale || '')}</span>` : '<span class="muted">—</span>'}</td>
                        <td class="${c.wash_sale_risk ? 'neg' : 'pos'}">${c.wash_sale_risk ? '⚠️ ' + esc(c.wash_sale_reason || 'yes') : '✓ none'}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        summary.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
