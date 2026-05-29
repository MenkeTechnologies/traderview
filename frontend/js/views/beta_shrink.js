// Vasicek (1973) Bayesian Beta Shrinkage view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseAssetsBlob, assetsToBlob, parseMarketBlob, marketToBlob,
    validateInputs, buildBody, localShrink,
    weightBadge, betaBadge, dispersionBadge,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPct, fmtInt,
} from '../_beta_shrink_inputs.js';

let state = { ...makeDemoInput('mixed') };

export async function renderBetaShrink(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.beta_shrink.h1.title" class="view-title">// BETA SHRINKAGE (Vasicek)</h1>

        <div class="chart-panel" data-context-scope="beta-shrinkage">
            <h2 data-i18n="view.beta_shrink.h2.market">Market returns
                <small data-i18n="view.beta_shrink.h2.market_hint" class="muted">(≥ 5 obs; benchmark/index)</small></h2>
            <textarea id="bs-market" rows="4"
                      data-tip="view.beta_shrink.tip.market"
                      placeholder="0.012, -0.004, 0.008, ...">${esc(marketToBlob(state.market_returns))}</textarea>

            <h2 data-i18n="view.beta_shrink.h2.assets">Asset returns
                <small data-i18n="view.beta_shrink.h2.assets_hint" class="muted">(one asset per line: SYMBOL r1 r2 r3 ...)</small></h2>
            <textarea id="bs-assets" rows="6"
                      data-tip="view.beta_shrink.tip.assets"
                      placeholder="LOW 0.005 -0.002 ...\nHIGH 0.015 -0.006 ...">${esc(assetsToBlob(state.assets))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.beta_shrink.btn.compute" id="bs-run" class="primary"
                        data-tip="view.beta_shrink.tip.compute" type="button">Shrink</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.beta_shrink.btn.demo_mixed"  id="bs-d1" class="secondary" type="button">Demo: mixed betas</button>
                <button data-i18n="view.beta_shrink.btn.demo_tight"  id="bs-d2" class="secondary" type="button">Demo: tight vs noisy</button>
                <button data-i18n="view.beta_shrink.btn.demo_sim"    id="bs-d3" class="secondary" type="button">Demo: all near 1.0</button>
                <button data-i18n="view.beta_shrink.btn.demo_sector" id="bs-d4" class="secondary" type="button">Demo: sector mix</button>
                <button data-i18n="view.beta_shrink.btn.demo_inv"    id="bs-d5" class="secondary" type="button">Demo: inverse ETFs</button>
                <button data-i18n="view.beta_shrink.btn.demo_short"  id="bs-d6" class="secondary" type="button">Demo: short series (n=10)</button>
                <button data-i18n="view.beta_shrink.btn.demo_mismatch" id="bs-d7" class="secondary" type="button">Demo: length mismatch</button>
                <button data-i18n="view.beta_shrink.btn.demo_single" id="bs-d8" class="secondary" type="button">Demo: single asset</button>
            </div>
            <p data-i18n="view.beta_shrink.hint.about" class="muted">Vasicek (1973) Bayes-optimal shrinkage: β̂ = w·β_OLS + (1−w)·β̄, w = σ²_cs/(σ²_cs + se²_OLS). Pulls noisy estimates toward cross-sectional mean β̄. High-se assets get strong shrinkage; tight-fit assets stay near OLS. Reduces estimation error vs raw OLS beta.</p>
        </div>

        <div id="bs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.beta_shrink.h2.table">Per-asset shrinkage</h2>
            <div id="bs-table"></div>
        </div>

        <div id="bs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bs-market').value = marketToBlob(state.market_returns);
        document.getElementById('bs-assets').value = assetsToBlob(state.assets);
    };
    document.getElementById('bs-d1').addEventListener('click', () => { loadDemo('mixed');           void compute(tok); });
    document.getElementById('bs-d2').addEventListener('click', () => { loadDemo('tight-vs-noisy'); void compute(tok); });
    document.getElementById('bs-d3').addEventListener('click', () => { loadDemo('all-similar');    void compute(tok); });
    document.getElementById('bs-d4').addEventListener('click', () => { loadDemo('sector-mix');     void compute(tok); });
    document.getElementById('bs-d5').addEventListener('click', () => { loadDemo('inverse');        void compute(tok); });
    document.getElementById('bs-d6').addEventListener('click', () => { loadDemo('short-series');   void compute(tok); });
    document.getElementById('bs-d7').addEventListener('click', () => { loadDemo('mismatched');     void compute(tok); });
    document.getElementById('bs-d8').addEventListener('click', () => { loadDemo('single');         void compute(tok); });
    document.getElementById('bs-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const mk = parseMarketBlob(document.getElementById('bs-market').value);
    const as = parseAssetsBlob(document.getElementById('bs-assets').value);
    const errs = [...mk.errors, ...as.errors];
    if (errs.length) {
        showErr(`${t('view.beta_shrink.err.parse_prefix')}: `
            + errs.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.market_returns = mk.market_returns;
    state.assets = as.assets;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localShrink(state.assets, state.market_returns);
    if (!local) { showErr(t('view.beta_shrink.err.degenerate')); return; }
    renderSummary(local, true);
    renderTable(local);
    let resp;
    try {
        resp = await api.anlyBetaShrinkage(buildBody(state));
    } catch (e) {
        showErr(`${t('view.beta_shrink.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.beta_shrink.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localShrink(state.assets, state.market_returns);
    let parityOk = !!local
        && Math.abs(local.prior_beta - report.prior_beta) < 1e-9
        && Math.abs(local.cross_sectional_variance - report.cross_sectional_variance) < 1e-9
        && local.assets.length === report.assets.length;
    if (parityOk) {
        for (let i = 0; i < local.assets.length; i++) {
            const a = local.assets[i], b = report.assets[i];
            if (a.symbol !== b.symbol
                || Math.abs(a.beta_ols - b.beta_ols) > 1e-9
                || Math.abs(a.beta_shrunk - b.beta_shrunk) > 1e-9
                || Math.abs(a.shrinkage_weight - b.shrinkage_weight) > 1e-9) {
                parityOk = false; break;
            }
        }
    }
    const dBadge = dispersionBadge(report.cross_sectional_variance, report.assets.length);
    const avgW = report.assets.length
        ? report.assets.reduce((s, a) => s + a.shrinkage_weight, 0) / report.assets.length : NaN;
    const wBadge = weightBadge(avgW);
    const localTag = pending ? ` (${t('view.beta_shrink.tag.local')})` : '';
    document.getElementById('bs-summary').innerHTML = [
        card(t('view.beta_shrink.card.prior_beta'),  fmtNum(report.prior_beta) + localTag),
        card(t('view.beta_shrink.card.cs_var'),      fmtNum(report.cross_sectional_variance, 6)),
        card(t('view.beta_shrink.card.cs_sd'),       fmtNum(Math.sqrt(report.cross_sectional_variance), 4)),
        card(t('view.beta_shrink.card.dispersion'),  t(dBadge.key), dBadge.cls),
        card(t('view.beta_shrink.card.n_assets'),    fmtInt(report.assets.length)),
        card(t('view.beta_shrink.card.n_obs'),       fmtInt(state.market_returns.length)),
        card(t('view.beta_shrink.card.avg_weight'),  fmtPct(avgW), wBadge.cls),
        card(t('view.beta_shrink.card.avg_strength'), t(wBadge.key), wBadge.cls),
        card(t('view.beta_shrink.card.parity'),
             parityOk ? t('view.beta_shrink.tag.ok') : t('view.beta_shrink.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(report) {
    const wrap = document.getElementById('bs-table');
    if (!report.assets || report.assets.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.beta_shrink.empty">${esc(t('view.beta_shrink.empty'))}</div>`;
        return;
    }
    const rows = report.assets.map(a => {
        const bB = betaBadge(a.beta_shrunk);
        const wB = weightBadge(a.shrinkage_weight);
        const movement = a.beta_shrunk - a.beta_ols;
        return `<tr>
            <td><strong>${esc(a.symbol)}</strong></td>
            <td>${esc(fmtNum(a.beta_ols))}</td>
            <td>${esc(fmtNum(a.standard_error))}</td>
            <td>${esc(fmtPct(a.shrinkage_weight))} <span class="${wB.cls}">${esc(t(wB.key))}</span></td>
            <td class="${bB.cls}">${esc(fmtNum(a.beta_shrunk))} <span>${esc(t(bB.key))}</span></td>
            <td class="${movement > 0 ? 'pos' : movement < 0 ? 'neg' : ''}">${esc(fmtNumSigned(movement))}</td>
        </tr>`;
    }).join('');
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.beta_shrink.col.symbol">Symbol</th>
                <th data-i18n="view.beta_shrink.col.beta_ols">β OLS</th>
                <th data-i18n="view.beta_shrink.col.se">SE(β)</th>
                <th data-i18n="view.beta_shrink.col.weight">Weight (w)</th>
                <th data-i18n="view.beta_shrink.col.beta_shrunk">β shrunk</th>
                <th data-i18n="view.beta_shrink.col.movement">Δβ</th>
            </tr></thead>
            <tbody>${rows}</tbody>
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
    const el = document.getElementById('bs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bs-err').style.display = 'none'; }
