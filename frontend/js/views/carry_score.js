// Carry-trade score view. (rate_long − rate_funding) / vol — Sharpe-style
// FX carry attractiveness score.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
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
                    <input id="cs-long" type="number" step="any" value="${state.long_rate}"></label>
                <label><span data-i18n="view.carry_score.label.funding_rate">Funding rate (decimal)</span>
                    <input id="cs-fund" type="number" step="any" value="${state.funding_rate}"></label>
                <label><span data-i18n="view.carry_score.label.vol">Annualized vol (decimal)</span>
                    <input id="cs-vol" type="number" step="any" min="0" value="${state.annualized_vol}"></label>
                <button data-i18n="view.carry_score.btn.score" id="cs-run" class="primary"
                        data-tip="view.carry_score.tip.score" type="button">Score</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.carry_score.btn.demo_strong"   id="cs-demo-strong"  class="secondary" type="button">Demo: MXN/JPY strong</button>
                <button data-i18n="view.carry_score.btn.demo_okay"     id="cs-demo-okay"    class="secondary" type="button">Demo: AUD/JPY okay</button>
                <button data-i18n="view.carry_score.btn.demo_poor"     id="cs-demo-poor"    class="secondary" type="button">Demo: poor (high-vol EM)</button>
                <button data-i18n="view.carry_score.btn.demo_negative" id="cs-demo-neg"     class="secondary" type="button">Demo: negative carry</button>
                <button data-i18n="view.carry_score.btn.demo_bstrong"  id="cs-demo-bs"      class="secondary" type="button">Demo: boundary 1.0 (strong)</button>
                <button data-i18n="view.carry_score.btn.demo_bokay"    id="cs-demo-bo"      class="secondary" type="button">Demo: boundary 0.5 (okay)</button>
                <button data-i18n="view.carry_score.btn.demo_zerovol"  id="cs-demo-zv"      class="secondary" type="button">Demo: zero-vol edge</button>
                <button data-i18n="view.carry_score.btn.demo_eur_usd"  id="cs-demo-eu"      class="secondary" type="button">Demo: EUR/USD 2024 (negative)</button>
            </div>
            <p data-i18n="view.carry_score.hint.about" class="muted">Score = (long − funding) / vol. ≥ 1.0 strong · ≥ 0.5 okay · &lt; 0.5 poor · negative differential overrides. Sharpe-like FX-carry attractiveness; ignores skew + jump risk.</p>
        </div>

        <div id="cs-summary" class="cards"></div>

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
    if (err) { showErr(err); return; }
    const local = localScore(state.long_rate, state.funding_rate, state.annualized_vol);
    renderSummary(local, true);
    let resp;
    try {
        resp = await api.calcCarryScore(buildBody(state));
    } catch (e) {
        showErr(`${t('view.carry_score.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
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
