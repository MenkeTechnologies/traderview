// Buying-power calculator view. Reg-T / PDT / portfolio margin / cash.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    ACCOUNT_TYPES, DEFAULT_INPUTS,
    validateInputs, buildBody, localCompute, dec,
    leverageBadge, pdtStatusKey, makeDemoInput,
    fmtUSD, fmtNum, fmtX, fmtPct,
} from '../_buying_power_inputs.js';

let state = { ...DEFAULT_INPUTS };

export async function renderBuyingPower(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.buying_power.h1.title" class="view-title">// BUYING POWER</h1>

        <div class="chart-panel" data-context-scope="buying-power">
            <h2 data-i18n="view.buying_power.h2.account">Account</h2>
            <div class="inline-form">
                <label><span data-i18n="view.buying_power.label.account_type">Account type</span>
                    <select id="bp-type">
                        ${ACCOUNT_TYPES.map(at => `<option value="${at}" ${state.account_type === at ? 'selected' : ''} data-i18n="view.buying_power.account.${at}">${at}</option>`).join('')}
                    </select></label>
                <label><span data-i18n="view.buying_power.label.equity">Equity ($)</span>
                    <input id="bp-eq" type="number" step="any" min="0" value="${state.equity}"></label>
                <label><span data-i18n="view.buying_power.label.share_price">Share price ($)</span>
                    <input id="bp-px" type="number" step="any" min="0" value="${state.share_price}"></label>
                <label><span data-i18n="view.buying_power.label.pdt">PDT flag?</span>
                    <input id="bp-pdt" type="checkbox" ${state.is_pdt ? 'checked' : ''}></label>
                <label><span data-i18n="view.buying_power.label.day_trade">Day-trade?</span>
                    <input id="bp-dt"  type="checkbox" ${state.is_day_trade ? 'checked' : ''}></label>
                <button data-i18n="view.buying_power.btn.compute" id="bp-run" class="primary"
                        data-tip="view.buying_power.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.buying_power.btn.demo_cash"      id="bp-demo-cash"  class="secondary" type="button">Demo: cash 1×</button>
                <button data-i18n="view.buying_power.btn.demo_regt"      id="bp-demo-regt"  class="secondary" type="button">Demo: Reg-T 2× overnight</button>
                <button data-i18n="view.buying_power.btn.demo_pdt"       id="bp-demo-pdt"   class="secondary" type="button">Demo: PDT day-trade 4×</button>
                <button data-i18n="view.buying_power.btn.demo_pdt_under" id="bp-demo-pu"    class="secondary" type="button">Demo: PDT flag but &lt;$25k → 2×</button>
                <button data-i18n="view.buying_power.btn.demo_pdt_over"  id="bp-demo-po"    class="secondary" type="button">Demo: PDT overnight → 2×</button>
                <button data-i18n="view.buying_power.btn.demo_sub5"      id="bp-demo-sub5"  class="secondary" type="button">Demo: sub-$5 → 1×</button>
                <button data-i18n="view.buying_power.btn.demo_pdt_sub5"  id="bp-demo-ps5"   class="secondary" type="button">Demo: PDT + sub-$5 → 4× (corner)</button>
                <button data-i18n="view.buying_power.btn.demo_pm"        id="bp-demo-pm"    class="secondary" type="button">Demo: portfolio margin 3×</button>
                <button data-i18n="view.buying_power.btn.demo_pm_pdt"    id="bp-demo-pmpdt" class="secondary" type="button">Demo: portfolio + PDT 6×</button>
            </div>
            <p data-i18n="view.buying_power.hint.about" class="muted">FINRA Rule 4210 + Reg-T. Sub-$5 stocks require 100% initial in standard Reg-T (no leverage on penny stocks) — but PDT day-trades override even that. Portfolio margin treated as ~3× overnight, 6× PDT-day.</p>
        </div>

        <div id="bp-summary" class="cards"></div>

        <div id="bp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bp-type').value = state.account_type;
        document.getElementById('bp-eq').value   = state.equity;
        document.getElementById('bp-px').value   = state.share_price;
        document.getElementById('bp-pdt').checked = state.is_pdt;
        document.getElementById('bp-dt').checked  = state.is_day_trade;
    };
    document.getElementById('bp-demo-cash').addEventListener('click',  () => loadDemo('cash'));
    document.getElementById('bp-demo-regt').addEventListener('click',  () => loadDemo('reg-t-overnight'));
    document.getElementById('bp-demo-pdt').addEventListener('click',   () => loadDemo('pdt-day-trade'));
    document.getElementById('bp-demo-pu').addEventListener('click',    () => loadDemo('pdt-below-25k'));
    document.getElementById('bp-demo-po').addEventListener('click',    () => loadDemo('pdt-overnight'));
    document.getElementById('bp-demo-sub5').addEventListener('click',  () => loadDemo('sub-5'));
    document.getElementById('bp-demo-ps5').addEventListener('click',   () => loadDemo('pdt-sub-5'));
    document.getElementById('bp-demo-pm').addEventListener('click',    () => loadDemo('portfolio-margin'));
    document.getElementById('bp-demo-pmpdt').addEventListener('click', () => loadDemo('pm-pdt-day'));
    document.getElementById('bp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    state = {
        account_type: document.getElementById('bp-type').value,
        equity:       Number(document.getElementById('bp-eq').value),
        share_price:  Number(document.getElementById('bp-px').value),
        is_pdt:       document.getElementById('bp-pdt').checked,
        is_day_trade: document.getElementById('bp-dt').checked,
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state);
    renderSummary({
        max_notional: local.max_notional, max_shares: local.max_shares,
        leverage: local.leverage, initial_requirement_pct: local.initial_requirement_pct,
        note: t(local.note_key),
    }, true);
    let resp;
    try {
        resp = await api.calcBuyingPower(buildBody(state));
    } catch (e) {
        showErr(`${t('view.buying_power.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary({
        ...resp,
        max_notional: dec(resp.max_notional),
        max_shares:   dec(resp.max_shares),
    }, false);
}

function renderSummary(report, pending) {
    const badge = leverageBadge(report.leverage);
    const local = localCompute(state);
    const parityOk = Math.abs(report.leverage - local.leverage) < 1e-9
                  && Math.abs(report.max_notional - local.max_notional) < 1e-6;
    const localTag = pending ? ` (${t('view.buying_power.tag.local')})` : '';
    document.getElementById('bp-summary').innerHTML = [
        card(t('view.buying_power.card.verdict'),
             t(badge.key) + localTag, badge.cls),
        card(t('view.buying_power.card.leverage'),
             fmtX(report.leverage),
             report.leverage >= 4 ? 'neg' : report.leverage >= 2 ? '' : 'pos'),
        card(t('view.buying_power.card.max_notional'),
             fmtUSD(report.max_notional),
             report.leverage >= 4 ? 'neg' : ''),
        card(t('view.buying_power.card.max_shares'),
             fmtNum(report.max_shares, 4)),
        card(t('view.buying_power.card.initial_req'),
             fmtPct(report.initial_requirement_pct),
             report.initial_requirement_pct >= 1 ? 'neg' : ''),
        card(t('view.buying_power.card.equity'),
             fmtUSD(state.equity)),
        card(t('view.buying_power.card.share_price'),
             fmtUSD(state.share_price)),
        card(t('view.buying_power.card.pdt_status'),
             t(pdtStatusKey(state)),
             state.is_pdt && state.is_day_trade && state.equity >= 25_000 ? 'neg' : ''),
        card(t('view.buying_power.card.note'),
             report.note || t(local.note_key)),
        card(t('view.buying_power.card.parity'),
             parityOk ? t('view.buying_power.tag.ok') : t('view.buying_power.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('bp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bp-err').style.display = 'none'; }
