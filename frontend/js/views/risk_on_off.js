// Risk-On / Risk-Off cross-asset regime view. Scores SPY/Gold/DXY/10Y
// snapshot into one of three regimes.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_INPUTS, validateInputs, buildBody, localEvaluate,
    signalBreakdown, regimeBadge, makeDemoSnap,
    fmtPctSigned, fmtBpsSigned, fmtScore,
    directionLabelKey, contributionClass,
} from '../_risk_on_off_inputs.js';

let state = { ...DEFAULT_INPUTS };

export async function renderRiskOnOff(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.risk_on_off.h1.title" class="view-title">// RISK-ON / RISK-OFF</h1>

        <div class="chart-panel" data-context-scope="risk-on-off">
            <h2 data-i18n="view.risk_on_off.h2.snapshot">Cross-asset snapshot</h2>
            <div class="inline-form">
                <label><span data-i18n="view.risk_on_off.label.spy">SPY change %  (decimal — 0.01 = +1%)</span>
                    <input id="ro-spy" type="number" step="any" value="${state.spy_change_pct}"></label>
                <label><span data-i18n="view.risk_on_off.label.gold">Gold change %</span>
                    <input id="ro-gold" type="number" step="any" value="${state.gold_change_pct}"></label>
                <label><span data-i18n="view.risk_on_off.label.dxy">DXY change %</span>
                    <input id="ro-dxy" type="number" step="any" value="${state.dxy_change_pct}"></label>
                <label><span data-i18n="view.risk_on_off.label.yields">10Y yield Δbps</span>
                    <input id="ro-yld" type="number" step="any" value="${state.ten_year_yield_bps_change}"></label>
                <button data-i18n="view.risk_on_off.btn.evaluate" id="ro-run" class="primary"
                        data-tip="view.risk_on_off.tip.evaluate" type="button">Evaluate</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.risk_on_off.btn.demo_full_on"   id="ro-demo-on"      class="secondary" type="button">Demo: FULL risk-ON</button>
                <button data-i18n="view.risk_on_off.btn.demo_full_off"  id="ro-demo-off"     class="secondary" type="button">Demo: FULL risk-OFF</button>
                <button data-i18n="view.risk_on_off.btn.demo_maj_off"   id="ro-demo-majoff"  class="secondary" type="button">Demo: majority off</button>
                <button data-i18n="view.risk_on_off.btn.demo_mixed"     id="ro-demo-mix"     class="secondary" type="button">Demo: mixed → neutral</button>
                <button data-i18n="view.risk_on_off.btn.demo_flat"      id="ro-demo-flat"    class="secondary" type="button">Demo: flat tape</button>
                <button data-i18n="view.risk_on_off.btn.demo_minority"  id="ro-demo-min"     class="secondary" type="button">Demo: 1 signal (still neutral)</button>
                <button data-i18n="view.risk_on_off.btn.demo_noisy"     id="ro-demo-noisy"   class="secondary" type="button">Demo: noisy SPY (still ON)</button>
                <button data-i18n="view.risk_on_off.btn.demo_bond"      id="ro-demo-bond"    class="secondary" type="button">Demo: bond rally (off)</button>
            </div>
            <p data-i18n="view.risk_on_off.hint.about" class="muted">Heuristic regime: SPY direction +1, Gold inverse +1, DXY inverse +1, Yields +1 (positive correlation). Score ≥ +2 = ON; ≤ -2 = OFF; else neutral. Noise floor 0.1% / ±1bp.</p>
        </div>

        <div id="ro-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk_on_off.h2.breakdown">Per-signal breakdown</h2>
            <div id="ro-breakdown"></div>
        </div>

        <div id="ro-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoSnap(k);
        document.getElementById('ro-spy').value  = state.spy_change_pct;
        document.getElementById('ro-gold').value = state.gold_change_pct;
        document.getElementById('ro-dxy').value  = state.dxy_change_pct;
        document.getElementById('ro-yld').value  = state.ten_year_yield_bps_change;
    };
    document.getElementById('ro-demo-on').addEventListener('click',     () => loadDemo('full-on'));
    document.getElementById('ro-demo-off').addEventListener('click',    () => loadDemo('full-off'));
    document.getElementById('ro-demo-majoff').addEventListener('click', () => loadDemo('majority-off'));
    document.getElementById('ro-demo-mix').addEventListener('click',    () => loadDemo('mixed-neutral'));
    document.getElementById('ro-demo-flat').addEventListener('click',   () => loadDemo('flat'));
    document.getElementById('ro-demo-min').addEventListener('click',    () => loadDemo('minority-on'));
    document.getElementById('ro-demo-noisy').addEventListener('click',  () => loadDemo('noisy-spy'));
    document.getElementById('ro-demo-bond').addEventListener('click',   () => loadDemo('bond-rally'));
    document.getElementById('ro-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    state = {
        spy_change_pct:  Number(document.getElementById('ro-spy').value),
        gold_change_pct: Number(document.getElementById('ro-gold').value),
        dxy_change_pct:  Number(document.getElementById('ro-dxy').value),
        ten_year_yield_bps_change: Number(document.getElementById('ro-yld').value),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localEvaluate(state);
    renderSummary(local, true);
    renderBreakdown();
    let resp;
    try {
        resp = await api.calcRiskOnOff(buildBody(state));
    } catch (e) {
        showErr(`${t('view.risk_on_off.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderBreakdown();
}

function renderSummary(report, pending) {
    const badge = regimeBadge(report.regime);
    const local = localEvaluate(state);
    const parityOk = report.regime === local.regime && report.score === local.score;
    const localTag = pending ? ` (${t('view.risk_on_off.tag.local')})` : '';
    document.getElementById('ro-summary').innerHTML = [
        card(t('view.risk_on_off.card.regime'),
             t(badge.key) + localTag, badge.cls),
        card(t('view.risk_on_off.card.score'),
             fmtScore(report.score),
             report.score > 0 ? 'pos' : report.score < 0 ? 'neg' : ''),
        card(t('view.risk_on_off.card.agreement'),
             `${report.agreement_count} / ${report.total_signals}`,
             report.agreement_count === report.total_signals ? 'pos' : ''),
        card(t('view.risk_on_off.card.noisy'),
             String(report.total_signals - report.agreement_count),
             (report.total_signals - report.agreement_count) > 0 ? 'neg' : ''),
        card(t('view.risk_on_off.card.parity'),
             parityOk ? t('view.risk_on_off.tag.ok') : t('view.risk_on_off.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderBreakdown() {
    const wrap = document.getElementById('ro-breakdown');
    const sigs = signalBreakdown(state);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.risk_on_off.col.signal">Signal</th>
                <th data-i18n="view.risk_on_off.col.value">Value</th>
                <th data-i18n="view.risk_on_off.col.direction">Direction</th>
                <th data-i18n="view.risk_on_off.col.contribution">Risk-on contribution</th>
            </tr></thead>
            <tbody>
                ${sigs.map(s => `<tr>
                    <td><strong data-i18n="view.risk_on_off.signal.${esc(s.name)}">${esc(s.name)}</strong></td>
                    <td>${esc(s.name === 'yields' ? fmtBpsSigned(s.value) : fmtPctSigned(s.value))}</td>
                    <td data-i18n="${esc(directionLabelKey(s.direction))}">${dirGlyph(s.direction)}</td>
                    <td class="${contributionClass(s.contribution)}">${fmtScore(s.contribution)}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function dirGlyph(d) {
    if (d > 0) return '▲';
    if (d < 0) return '▼';
    return '·';
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('ro-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ro-err').style.display = 'none'; }
