// Commission optimizer view — compares your real execution profile against
// alternative broker tiers. Reports projected annual savings if you switch.
//
// All user-facing strings flow through t() / data-i18n.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import {
    parseExecutionBlob, validateInputs, buildBody, localEvaluate, dec,
    defaultTiers, makeDemoExecutions, savingsBadge,
    fmtUSD, fmtUSDSigned, fmtPct, fmtInt,
} from '../_commission_optimizer_inputs.js';

let state = {
    executions: makeDemoExecutions('active-retail'),
    tiers: defaultTiers(),
};

export async function renderCommissionOptimizer(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.commission_optimizer.h1.title" class="view-title">// COMMISSION OPTIMIZER</h1>

        <div class="chart-panel" data-context-scope="commission-optimizer">
            <h2 data-i18n="view.commission_optimizer.h2.executions">Executions
                <small data-i18n="view.commission_optimizer.h2.executions_hint" class="muted">(per line: qty notional actual_fee)</small></h2>
            <textarea id="co-execs" rows="6" placeholder="100 5000 1.00&#10;200 8000 1.00&#10;...">${esc(execsToBlob(state.executions))}</textarea>

            <h2 data-i18n="view.commission_optimizer.h2.tiers">Tiers (JSON)
                <small data-i18n="view.commission_optimizer.label.tiers_hint" class="muted">JSON array of {name, per_trade_flat, per_share, per_dollar, min_per_trade, max_per_trade}</small></h2>
            <textarea id="co-tiers" rows="6">${esc(tiersToJson(state.tiers))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.commission_optimizer.btn.evaluate" id="co-run" class="primary" type="button">Evaluate</button>
                <button data-i18n="view.commission_optimizer.btn.reset_tiers" id="co-reset" class="secondary" type="button">Reset to default tiers (IBKR / Lightspeed / Webull)</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.commission_optimizer.btn.demo_active"  id="co-demo-active"  class="secondary" type="button">Demo: active retail</button>
                <button data-i18n="view.commission_optimizer.btn.demo_scalper" id="co-demo-scalper" class="secondary" type="button">Demo: heavy scalper</button>
                <button data-i18n="view.commission_optimizer.btn.demo_options" id="co-demo-options" class="secondary" type="button">Demo: light options</button>
                <button data-i18n="view.commission_optimizer.btn.demo_webull"  id="co-demo-webull"  class="secondary" type="button">Demo: already on Webull</button>
                <button data-i18n="view.commission_optimizer.btn.demo_blocks"  id="co-demo-blocks"  class="secondary" type="button">Demo: big blocks</button>
            </div>
            <p data-i18n="view.commission_optimizer.hint.about" class="muted">Compares your actual commission spend against alternative tier pricing on your real execution profile. Annual savings = best alternative × 12.</p>
        </div>

        <div id="co-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.commission_optimizer.h2.results">Per-tier comparison (sorted cheapest first)</h2>
            <div id="co-table"></div>
        </div>

        <div id="co-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('co-demo-active').addEventListener('click',  () => loadDemo('active-retail'));
    document.getElementById('co-demo-scalper').addEventListener('click', () => loadDemo('scalper-heavy'));
    document.getElementById('co-demo-options').addEventListener('click', () => loadDemo('options-light'));
    document.getElementById('co-demo-webull').addEventListener('click',  () => loadDemo('webull-zero'));
    document.getElementById('co-demo-blocks').addEventListener('click',  () => loadDemo('big-blocks'));
    document.getElementById('co-reset').addEventListener('click', () => {
        state.tiers = defaultTiers();
        document.getElementById('co-tiers').value = tiersToJson(state.tiers);
        showToast(t('view.commission_optimizer.toast.saved'), { level: 'success', duration: 1500 });
    });
    document.getElementById('co-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function execsToBlob(execs) {
    return execs.map(e => `${e.qty} ${e.notional} ${e.actual_fee}`).join('\n');
}

function tiersToJson(tiers) {
    return JSON.stringify(tiers, null, 2);
}

function loadDemo(kind) {
    state.executions = makeDemoExecutions(kind);
    document.getElementById('co-execs').value = execsToBlob(state.executions);
}

function readInputs() {
    const parsed = parseExecutionBlob(document.getElementById('co-execs').value);
    if (parsed.errors.length) {
        showErr(`${t('view.commission_optimizer.err.parse_prefix')}: `
            + parsed.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    state.executions = parsed.executions;
    try {
        const tiers = JSON.parse(document.getElementById('co-tiers').value);
        if (!Array.isArray(tiers)) throw new Error(t('view.commission_optimizer.error.tiers_array'));
        for (const t_ of tiers) {
            for (const k of ['per_trade_flat', 'per_share', 'per_dollar', 'min_per_trade', 'max_per_trade']) {
                t_[k] = Number(t_[k]);
            }
        }
        state.tiers = tiers;
    } catch (e) {
        showErr(`${t('view.commission_optimizer.err.tiers_json')}: ${e.message}`);
        return;
    }
    hideErr();
}

async function compute(tok) {
    const err = validateInputs(state.executions, state.tiers);
    if (err) { showErr(err); return; }
    hideErr();
    const local = localEvaluate(state.executions, state.tiers);
    renderSummary(local, true);
    renderTable(local);
    let resp;
    try {
        resp = await api.calcCommissionOptimizer(buildBody(state.executions, state.tiers));
    } catch (e) {
        showErr(`${t('view.commission_optimizer.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    const normalized = {
        ...resp,
        total_shares:           dec(resp.total_shares),
        total_notional:         dec(resp.total_notional),
        actual_total_fee:       dec(resp.actual_total_fee),
        projected_annual_savings: dec(resp.projected_annual_savings),
        tiers: (resp.tiers || []).map(tr => ({
            ...tr,
            total_fee:        dec(tr.total_fee),
            fee_per_trade:    dec(tr.fee_per_trade),
            fee_per_share:    dec(tr.fee_per_share),
            delta_vs_actual:  dec(tr.delta_vs_actual),
        })),
    };
    renderSummary(normalized, false);
    renderTable(normalized);
}

function renderSummary(report, pending) {
    const badge = savingsBadge(report.projected_annual_savings);
    const local = localEvaluate(state.executions, state.tiers);
    const parity = Math.abs(report.actual_total_fee - local.actual_total_fee) < 1e-6
                && (report.best_alternative || null) === (local.best_alternative || null);
    const localTag = pending ? ` (${t('view.commission_optimizer.tag.local')})` : '';
    const actualPerTrade = report.trade_count > 0
        ? report.actual_total_fee / report.trade_count : 0;
    const actualPerShare = report.total_shares > 0
        ? report.actual_total_fee / report.total_shares : 0;
    document.getElementById('co-summary').innerHTML = [
        card(t('view.commission_optimizer.card.verdict'),
             t(badge.key) + localTag, badge.cls || ''),
        card(t('view.commission_optimizer.card.best_alt'),
             report.best_alternative || t('view.commission_optimizer.tag.none'),
             report.best_alternative ? 'pos' : ''),
        card(t('view.commission_optimizer.card.annual_save'),
             fmtUSD(report.projected_annual_savings),
             report.projected_annual_savings > 0 ? 'pos' : ''),
        card(t('view.commission_optimizer.card.trade_count'),
             fmtInt(report.trade_count)),
        card(t('view.commission_optimizer.card.total_shares'),
             fmtInt(report.total_shares)),
        card(t('view.commission_optimizer.card.total_notional'),
             fmtUSD(report.total_notional)),
        card(t('view.commission_optimizer.card.actual_fee'),
             fmtUSD(report.actual_total_fee), 'neg'),
        card(t('view.commission_optimizer.card.fee_per_trade'),
             fmtUSD(actualPerTrade)),
        card(t('view.commission_optimizer.card.fee_per_share'),
             '$' + actualPerShare.toFixed(5)),
        card(t('view.commission_optimizer.card.parity'),
             parity ? t('view.commission_optimizer.tag.ok')
                    : t('view.commission_optimizer.tag.diverged'),
             parity ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(report) {
    const wrap = document.getElementById('co-table');
    if (!report.tiers || report.tiers.length === 0) {
        wrap.innerHTML = `<div class="muted">${esc(t('view.commission_optimizer.tag.none'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.commission_optimizer.col.tier">Tier</th>
                <th data-i18n="view.commission_optimizer.col.total">Total fee</th>
                <th data-i18n="view.commission_optimizer.col.per_trade">/ Trade</th>
                <th data-i18n="view.commission_optimizer.col.per_share">/ Share</th>
                <th data-i18n="view.commission_optimizer.col.pct_notional">% of notional</th>
                <th data-i18n="view.commission_optimizer.col.delta">Δ vs actual</th>
            </tr></thead>
            <tbody>
                ${report.tiers.map((tr, i) => `<tr class="${i === 0 ? 'pos' : ''}">
                    <td><strong>${esc(tr.tier)}</strong></td>
                    <td class="neg">${esc(fmtUSD(tr.total_fee))}</td>
                    <td>${esc(fmtUSD(tr.fee_per_trade))}</td>
                    <td>$${tr.fee_per_share.toFixed(5)}</td>
                    <td>${esc(fmtPct(tr.fee_pct_of_notional))}</td>
                    <td class="${tr.delta_vs_actual < 0 ? 'pos' : tr.delta_vs_actual > 0 ? 'neg' : ''}">${esc(fmtUSDSigned(tr.delta_vs_actual))}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('co-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('co-err').style.display = 'none'; }
