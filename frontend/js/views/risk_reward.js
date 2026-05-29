// Risk-Reward calculator view. Given entry / stop / target / risk budget
// / multiplier, computes R:R, sizing qty, dollar risk/reward, breakeven
// win-rate, and a 3-step scale-out plan.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_INPUTS, validateInputs, buildBody, localCompute, dec,
    rrBadge, makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtNum, fmtPct, fmtR, fmtFraction,
} from '../_risk_reward_inputs.js';

let state = { ...DEFAULT_INPUTS };

export async function renderRiskReward(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.risk_reward.h1.title" class="view-title">// RISK / REWARD</h1>

        <div class="chart-panel" data-context-scope="risk-reward">
            <h2 data-i18n="view.risk_reward.h2.trade">Planned trade</h2>
            <div class="inline-form">
                <label><span data-i18n="view.risk_reward.label.side">Side</span>
                    <select id="rr-side">
                        <option value="long"  ${state.side === 'long'  ? 'selected' : ''} data-i18n="view.risk_reward.option.long">Long</option>
                        <option value="short" ${state.side === 'short' ? 'selected' : ''} data-i18n="view.risk_reward.option.short">Short</option>
                    </select></label>
                <label><span data-i18n="view.risk_reward.label.entry">Entry</span>
                    <input id="rr-entry" type="number" step="any" min="0" value="${state.entry}"></label>
                <label><span data-i18n="view.risk_reward.label.stop">Stop</span>
                    <input id="rr-stop" type="number" step="any" min="0" value="${state.stop}"></label>
                <label><span data-i18n="view.risk_reward.label.target">Target</span>
                    <input id="rr-target" type="number" step="any" min="0" value="${state.target}"></label>
                <label><span data-i18n="view.risk_reward.label.risk_budget">Risk budget ($)</span>
                    <input id="rr-budget" type="number" step="any" min="0" value="${state.risk_budget}"></label>
                <label><span data-i18n="view.risk_reward.label.multiplier">Multiplier (100 for options, 50 for ES, …)</span>
                    <input id="rr-mult" type="number" step="any" min="0" value="${state.multiplier}"></label>
                <button data-i18n="view.risk_reward.btn.compute" id="rr-run" class="primary"
                        data-tip="view.risk_reward.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.risk_reward.btn.demo_long_3r"  id="rr-demo-l3"  class="secondary" type="button">Demo: long 3R</button>
                <button data-i18n="view.risk_reward.btn.demo_long_1r"  id="rr-demo-l1"  class="secondary" type="button">Demo: long 1R (coin flip)</button>
                <button data-i18n="view.risk_reward.btn.demo_long_5r"  id="rr-demo-l5"  class="secondary" type="button">Demo: long 5R</button>
                <button data-i18n="view.risk_reward.btn.demo_short_3r" id="rr-demo-s3"  class="secondary" type="button">Demo: short 3R</button>
                <button data-i18n="view.risk_reward.btn.demo_options"  id="rr-demo-opt" class="secondary" type="button">Demo: 1 option (×100)</button>
                <button data-i18n="view.risk_reward.btn.demo_es"       id="rr-demo-es"  class="secondary" type="button">Demo: ES futures (×50)</button>
                <button data-i18n="view.risk_reward.btn.demo_bad_long" id="rr-demo-bl"  class="secondary" type="button">Demo: BAD long (target below)</button>
                <button data-i18n="view.risk_reward.btn.demo_bad_short" id="rr-demo-bs" class="secondary" type="button">Demo: BAD short (target above)</button>
                <button data-i18n="view.risk_reward.btn.demo_zero_stop" id="rr-demo-zs" class="secondary" type="button">Demo: zero stop dist</button>
            </div>
            <p data-i18n="view.risk_reward.hint.about" class="muted">qty = risk_budget / (stop_distance × multiplier). Breakeven win-rate = 1 / (1 + R). Scale-out plan: 1/3 at 1R, 1/3 at 2R, 1/3 at target.</p>
        </div>

        <div id="rr-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_reward.h2.scale_outs">Scale-out plan</h2>
            <div id="rr-table"></div>
        </div>

        <div id="rr-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('rr-side').value   = state.side;
        document.getElementById('rr-entry').value  = state.entry;
        document.getElementById('rr-stop').value   = state.stop;
        document.getElementById('rr-target').value = state.target;
        document.getElementById('rr-budget').value = state.risk_budget;
        document.getElementById('rr-mult').value   = state.multiplier;
    };
    document.getElementById('rr-demo-l3').addEventListener('click',  () => loadDemo('long-3r'));
    document.getElementById('rr-demo-l1').addEventListener('click',  () => loadDemo('long-1r'));
    document.getElementById('rr-demo-l5').addEventListener('click',  () => loadDemo('long-5r'));
    document.getElementById('rr-demo-s3').addEventListener('click',  () => loadDemo('short-3r'));
    document.getElementById('rr-demo-opt').addEventListener('click', () => loadDemo('options-1ct'));
    document.getElementById('rr-demo-es').addEventListener('click',  () => loadDemo('es-futures'));
    document.getElementById('rr-demo-bl').addEventListener('click',  () => loadDemo('broken-long'));
    document.getElementById('rr-demo-bs').addEventListener('click',  () => loadDemo('broken-short'));
    document.getElementById('rr-demo-zs').addEventListener('click',  () => loadDemo('zero-stop'));
    document.getElementById('rr-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    state = {
        side:        document.getElementById('rr-side').value,
        entry:       Number(document.getElementById('rr-entry').value),
        stop:        Number(document.getElementById('rr-stop').value),
        target:      Number(document.getElementById('rr-target').value),
        risk_budget: Number(document.getElementById('rr-budget').value),
        multiplier:  Number(document.getElementById('rr-mult').value),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state);
    if (!local.ok) {
        showErr(`${t('view.risk_reward.err.geometry')}: ${local.error}`);
        renderSummary(null, null, true);
        renderTable(null);
        return;
    }
    renderSummary(local.report, null, true);
    renderTable(local.report);
    let resp;
    try {
        resp = await api.calcRiskReward(buildBody(state));
    } catch (e) {
        showErr(`${t('view.risk_reward.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    const normalized = {
        ...resp,
        qty:           dec(resp.qty),
        dollar_risk:   dec(resp.dollar_risk),
        dollar_reward: dec(resp.dollar_reward),
        scale_outs: (resp.scale_outs || []).map(s => ({
            ...s, price: dec(s.price),
        })),
    };
    renderSummary(normalized, local.report, false);
    renderTable(normalized);
}

function renderSummary(report, localRef, pending) {
    if (!report) {
        document.getElementById('rr-summary').innerHTML = `
            <div class="card"><div class="label" data-i18n="view.risk_reward.card.verdict">${esc(t('view.risk_reward.card.verdict'))}</div>
                 <div class="value neg" data-i18n="view.risk_reward.tag.invalid">${esc(t('view.risk_reward.tag.invalid'))}</div></div>`;
        return;
    }
    const badge = rrBadge(report.r_multiple);
    const localTag = pending ? ` (${t('view.risk_reward.tag.local')})` : '';
    const parityOk = !localRef ? true
        : Math.abs(report.r_multiple - localRef.r_multiple) < 1e-9
          && Math.abs(report.qty - localRef.qty) < 1e-6;
    document.getElementById('rr-summary').innerHTML = [
        card(t('view.risk_reward.card.verdict'),  t(badge.key) + localTag, badge.cls),
        card(t('view.risk_reward.card.r_multiple'), fmtR(report.r_multiple),
             report.r_multiple >= 2 ? 'pos' : report.r_multiple < 1 ? 'neg' : ''),
        card(t('view.risk_reward.card.qty'),      fmtNum(report.qty, 4)),
        card(t('view.risk_reward.card.risk'),     fmtUSD(report.dollar_risk), 'neg'),
        card(t('view.risk_reward.card.reward'),   fmtUSD(report.dollar_reward), 'pos'),
        card(t('view.risk_reward.card.net'),      fmtUSDSigned(report.dollar_reward - report.dollar_risk),
             report.dollar_reward > report.dollar_risk ? 'pos' : 'neg'),
        card(t('view.risk_reward.card.breakeven_wr'), fmtPct(report.breakeven_win_rate),
             report.breakeven_win_rate < 0.5 ? 'pos' : 'neg'),
        card(t('view.risk_reward.card.parity'),   parityOk ? t('view.risk_reward.tag.ok') : t('view.risk_reward.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(report) {
    const wrap = document.getElementById('rr-table');
    if (!report || !report.scale_outs || report.scale_outs.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.risk_reward.empty">${esc(t('view.risk_reward.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.risk_reward.col.label">Level</th>
                <th data-i18n="view.risk_reward.col.price">Price</th>
                <th data-i18n="view.risk_reward.col.fraction">Fraction</th>
                <th data-i18n="view.risk_reward.col.qty_out">Qty exiting</th>
            </tr></thead>
            <tbody>
                ${report.scale_outs.map(s => `<tr>
                    <td><strong>${esc(s.label)}</strong></td>
                    <td>${esc(fmtNum(s.price, 4))}</td>
                    <td>${esc(fmtFraction(s.fraction))}</td>
                    <td>${esc(fmtNum(report.qty * s.fraction, 4))}</td>
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
    const el = document.getElementById('rr-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('rr-err').style.display = 'none'; }
