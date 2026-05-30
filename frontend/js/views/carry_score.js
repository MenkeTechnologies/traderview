// Carry-trade score view. (rate_long − rate_funding) / vol — Sharpe-style
// FX carry attractiveness score.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_INPUTS, validateInputs, buildBody, localScore,
    tierBadge, noteKeyForTier, makeDemoInput,
    fmtPct, fmtPctSigned, fmtScore,
} from '../_carry_score_inputs.js';

let state = { ...DEFAULT_INPUTS };

export async function renderCarryScore(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.carry_score.h1.title" class="view-title">// CARRY SCORE</h1>

        <div class="chart-panel" data-context-scope="carry-score">
            <h2 data-i18n="view.carry_score.h2.pair">Pair definition</h2>
            <div class="inline-form">
                <label><span data-i18n="view.carry_score.label.long_rate">Long rate (decimal — 0.05 = 5%)</span>
                    <input id="cs-long" type="number" step="any" value="${state.long_rate}" data-tip="view.carry_score.tip.long_rate"></label>
                <label><span data-i18n="view.carry_score.label.funding_rate">Funding rate (decimal)</span>
                    <input id="cs-fund" type="number" step="any" value="${state.funding_rate}" data-tip="view.carry_score.tip.funding_rate"></label>
                <label><span data-i18n="view.carry_score.label.vol">Annualized vol (decimal)</span>
                    <input id="cs-vol" type="number" step="any" min="0" value="${state.annualized_vol}" data-tip="view.carry_score.tip.vol"></label>
                <button data-i18n="view.carry_score.btn.score" id="cs-run" class="primary"
                        data-tip="view.carry_score.tip.score" data-shortcut="carry_score_run" type="button">Score</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.carry_score.btn.demo_strong"   id="cs-demo-strong"  class="secondary" type="button" data-tip="view.carry_score.tip.demo_strong">Demo: MXN/JPY strong</button>
                <button data-i18n="view.carry_score.btn.demo_okay"     id="cs-demo-okay"    class="secondary" type="button" data-tip="view.carry_score.tip.demo_okay">Demo: AUD/JPY okay</button>
                <button data-i18n="view.carry_score.btn.demo_poor"     id="cs-demo-poor"    class="secondary" type="button" data-tip="view.carry_score.tip.demo_poor">Demo: poor (high-vol EM)</button>
                <button data-i18n="view.carry_score.btn.demo_negative" id="cs-demo-neg"     class="secondary" type="button" data-tip="view.carry_score.tip.demo_neg">Demo: negative carry</button>
                <button data-i18n="view.carry_score.btn.demo_bstrong"  id="cs-demo-bs"      class="secondary" type="button" data-tip="view.carry_score.tip.demo_bs">Demo: boundary 1.0 (strong)</button>
                <button data-i18n="view.carry_score.btn.demo_bokay"    id="cs-demo-bo"      class="secondary" type="button" data-tip="view.carry_score.tip.demo_bo">Demo: boundary 0.5 (okay)</button>
                <button data-i18n="view.carry_score.btn.demo_zerovol"  id="cs-demo-zv"      class="secondary" type="button" data-tip="view.carry_score.tip.demo_zv">Demo: zero-vol edge</button>
                <button data-i18n="view.carry_score.btn.demo_eur_usd"  id="cs-demo-eu"      class="secondary" type="button" data-tip="view.carry_score.tip.demo_eu">Demo: EUR/USD 2024 (negative)</button>
            </div>
            <p data-i18n="view.carry_score.hint.about" class="muted">Score = (long − funding) / vol. ≥ 1.0 strong · ≥ 0.5 okay · &lt; 0.5 poor · negative differential overrides. Sharpe-like FX-carry attractiveness; ignores skew + jump risk.</p>
        </div>

        <div id="cs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.carry_score.h2.vol_chart">Score sensitivity to annualized vol</h2>
            <div id="cs-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.carry_score.h2.diff_chart">Score sensitivity to rate differential (fixed vol)</h2>
            <div id="cs-diff-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="cs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('cs-long').value = state.long_rate;
        document.getElementById('cs-fund').value = state.funding_rate;
        document.getElementById('cs-vol').value  = state.annualized_vol;
    };
    document.getElementById('cs-demo-strong').addEventListener('click', () => loadDemo('strong-mxn-jpy'));
    document.getElementById('cs-demo-okay').addEventListener('click',   () => loadDemo('okay-aud-jpy'));
    document.getElementById('cs-demo-poor').addEventListener('click',   () => loadDemo('poor-high-vol'));
    document.getElementById('cs-demo-neg').addEventListener('click',    () => loadDemo('negative-anti-carry'));
    document.getElementById('cs-demo-bs').addEventListener('click',     () => loadDemo('boundary-strong'));
    document.getElementById('cs-demo-bo').addEventListener('click',     () => loadDemo('boundary-okay'));
    document.getElementById('cs-demo-zv').addEventListener('click',     () => loadDemo('zero-vol'));
    document.getElementById('cs-demo-eu').addEventListener('click',     () => loadDemo('eur-vs-usd-2024'));
    document.getElementById('cs-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    state = {
        long_rate:      Number(document.getElementById('cs-long').value),
        funding_rate:   Number(document.getElementById('cs-fund').value),
        annualized_vol: Number(document.getElementById('cs-vol').value),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.carry_score.toast.invalid'), { level: 'warning' }); return; }
    const local = localScore(state.long_rate, state.funding_rate, state.annualized_vol);
    renderSummary(local, true);
    let resp;
    try {
        resp = await api.calcCarryScore(buildBody(state));
    } catch (e) {
        showErr(`${t('view.carry_score.err.api')}: ${e.message || e}`);
        showToast(t('view.carry_score.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderVolChart();
    renderDiffChart();
    const tier = String(resp.tier || '');
    const score = Number(resp.carry_score || 0).toFixed(2);
    const level = tier === 'strong' ? 'success' : tier === 'okay' ? 'info' : 'warning';
    showToast(t('view.carry_score.toast.scored', { tier, score }), { level });
}

function renderVolChart() {
    const el = document.getElementById('cs-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const diff = state.long_rate - state.funding_rate;
    if (!Number.isFinite(diff)) {
        el.innerHTML = `<div class="muted" data-i18n="view.carry_score.empty_chart">${esc(t('view.carry_score.empty_chart'))}</div>`;
        return;
    }
    const xs = [];
    const ys = [];
    const strong = [];
    const okay = [];
    for (let v = 0.02; v <= 0.40 + 1e-9; v += 0.005) {
        const score = v > 0 ? diff / v : null;
        xs.push(v);
        ys.push(score);
        strong.push(1.0);
        okay.push(0.5);
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: t('view.carry_score.chart.vol') },
            { label: t('view.carry_score.chart.score'),
              stroke: '#00e5ff', width: 1.6,
              points: { show: false } },
            { label: t('view.carry_score.chart.strong'),
              stroke: '#7af0a8', width: 1.0, dash: [4, 4],
              points: { show: false } },
            { label: t('view.carry_score.chart.okay'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 } ],
        legend: { show: true },
    }, [xs, ys, strong, okay], el);
}

function renderDiffChart() {
    const el = document.getElementById('cs-diff-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const vol = state.annualized_vol;
    if (!Number.isFinite(vol) || vol <= 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.carry_score.empty_diff_chart">${esc(t('view.carry_score.empty_diff_chart'))}</div>`;
        return;
    }
    const xs = [];
    const ys = [];
    const strong = [];
    const okay = [];
    const negStrong = [];
    for (let d = -0.10; d <= 0.10 + 1e-9; d += 0.005) {
        xs.push(d);
        ys.push(d / vol);
        strong.push(1.0);
        okay.push(0.5);
        negStrong.push(-1.0);
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: t('view.carry_score.chart.diff') },
            { label: t('view.carry_score.chart.score'),
              stroke: '#7af0a8', width: 1.6,
              points: { show: false } },
            { label: t('view.carry_score.chart.strong'),
              stroke: '#7af0a8', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('view.carry_score.chart.okay'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('view.carry_score.chart.neg_strong'),
              stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 } ],
        legend: { show: true },
    }, [xs, ys, strong, okay, negStrong], el);
}

function renderSummary(report, pending) {
    const badge = tierBadge(report.tier);
    const local = localScore(state.long_rate, state.funding_rate, state.annualized_vol);
    const parityOk = Math.abs(report.carry_score - local.carry_score) < 1e-9
                  && report.tier === local.tier;
    const localTag = pending ? ` (${t('view.carry_score.tag.local')})` : '';
    document.getElementById('cs-summary').innerHTML = [
        card(t('view.carry_score.card.verdict'),    t(badge.key) + localTag, badge.cls),
        card(t('view.carry_score.card.note'),       t(noteKeyForTier(report.tier))),
        card(t('view.carry_score.card.score'),      fmtScore(report.carry_score), badge.cls),
        card(t('view.carry_score.card.differential'),
             fmtPctSigned(report.rate_differential),
             report.rate_differential >= 0 ? 'pos' : 'neg'),
        card(t('view.carry_score.card.long_rate'),  fmtPct(report.long_rate)),
        card(t('view.carry_score.card.funding_rate'), fmtPct(report.funding_rate)),
        card(t('view.carry_score.card.vol'),        fmtPct(report.annualized_vol)),
        card(t('view.carry_score.card.parity'),
             parityOk ? t('view.carry_score.tag.ok') : t('view.carry_score.tag.diverged'),
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
    const el = document.getElementById('cs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cs-err').style.display = 'none'; }
