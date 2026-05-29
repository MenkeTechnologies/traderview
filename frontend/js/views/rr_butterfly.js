// Risk-Reversal / Butterfly Calculator — bidirectional FX vol-quote
// converter. Two modes share one backend endpoint:
//
//   Decompose:    input σ_25C / σ_25P / σ_ATM → ATM, RR, BF, skew z.
//   Reconstruct:  input ATM / RR / BF         → σ_25C, σ_25P.
//
// Useful for:
//   * "I see ATM=8%, 25-RR=-1.2%, 25-BF=0.3% on the broker screen —
//      what are the wing IVs that prices the BS calls/puts?" (recon)
//   * "I have three IVs from my marker — what skew does that imply
//      and how does it compare to ATM?" (decompose)

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    buildBody, validateInputs,
    decomposeLocal, reconstructLocal,
    fmtVolPct, fmtSkewZ,
} from '../_rr_butterfly_inputs.js';

import { t } from '../i18n.js';
const DEFAULTS = {
    decompose:   { sigma_25_call: 0.085, sigma_25_put: 0.097, sigma_atm: 0.090 },
    reconstruct: { atm: 0.090, rr: -0.012, bf: 0.001 },
};

let state = { mode: 'decompose', params: { ...DEFAULTS.decompose } };

export async function renderRrButterfly(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.rr_butterfly.h1.rr_bf_calculator" class="view-title">// RR / BF CALCULATOR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.rr_butterfly.h2.mode">Mode</h2>
            <div class="inline-form">
                <label><span data-i18n="view.rr_butterfly.label.direction">Direction</span>
                    <select id="rr-mode">
                        <option data-i18n="view.rr_butterfly.opt.decompose_atm_rr_bf" value="decompose"   ${state.mode === 'decompose'   ? 'selected' : ''}>Decompose → ATM + RR + BF</option>
                        <option data-i18n="view.rr_butterfly.opt.reconstruct_wings" value="reconstruct" ${state.mode === 'reconstruct' ? 'selected' : ''}>Reconstruct → σ wings</option>
                    </select></label>
                <button data-i18n="view.rr_butterfly.btn.compute" id="rr-run" class="primary" type="button">Compute</button>
            </div>
            <p data-i18n="view.rr_butterfly.hint.fx_vol_quotes_are_conventionally_in_vol_points_per" class="muted">
                FX vol quotes are conventionally in vol-points (percent). RR &gt; 0 = calls
                priced richer than puts (right tail fear); BF &gt; 0 = wings priced richer
                than ATM (smile curvature premium).
            </p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.rr_butterfly.h2.inputs">Inputs</h2>
            <div id="rr-inputs" class="inline-form"></div>
        </div>

        <div id="rr-summary" class="cards"></div>

        <div id="rr-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    renderInputsForm();
    wireForm(mount, tok);
    void fmt;
}

function renderInputsForm() {
    const wrap = document.getElementById('rr-inputs');
    if (state.mode === 'decompose') {
        wrap.innerHTML = `
            <label><span data-i18n="view.rr_butterfly.label.sigma_25c">σ_25C (call wing)</span>
                <input id="rr-sc"  type="number" step="any" min="0" value="${state.params.sigma_25_call}"></label>
            <label><span data-i18n="view.rr_butterfly.label.sigma_25p">σ_25P (put wing)</span>
                <input id="rr-sp"  type="number" step="any" min="0" value="${state.params.sigma_25_put}"></label>
            <label><span data-i18n="view.rr_butterfly.label.sigma_atm">σ_ATM</span>
                <input id="rr-atm" type="number" step="any" min="0" value="${state.params.sigma_atm}"></label>
        `;
        wrap.querySelector('#rr-sc').addEventListener('change',  e => state.params.sigma_25_call = Number(e.target.value));
        wrap.querySelector('#rr-sp').addEventListener('change',  e => state.params.sigma_25_put  = Number(e.target.value));
        wrap.querySelector('#rr-atm').addEventListener('change', e => state.params.sigma_atm     = Number(e.target.value));
    } else {
        wrap.innerHTML = `
            <label><span data-i18n="view.rr_butterfly.label.atm_iv">ATM IV</span>
                <input id="rr-atm-in" type="number" step="any" min="0" value="${state.params.atm}"></label>
            <label><span data-i18n="view.rr_butterfly.label.rr">Risk reversal (RR)</span>
                <input id="rr-rr"     type="number" step="any" value="${state.params.rr}"></label>
            <label><span data-i18n="view.rr_butterfly.label.bf">Butterfly (BF)</span>
                <input id="rr-bf"     type="number" step="any" value="${state.params.bf}"></label>
        `;
        wrap.querySelector('#rr-atm-in').addEventListener('change', e => state.params.atm = Number(e.target.value));
        wrap.querySelector('#rr-rr').addEventListener('change',     e => state.params.rr  = Number(e.target.value));
        wrap.querySelector('#rr-bf').addEventListener('change',     e => state.params.bf  = Number(e.target.value));
    }
}

function wireForm(mount, tok) {
    document.getElementById('rr-mode').addEventListener('change', e => {
        state.mode = e.target.value;
        state.params = { ...DEFAULTS[state.mode] };
        renderInputsForm();
        document.getElementById('rr-summary').innerHTML = '';
    });
    document.getElementById('rr-run').addEventListener('click', () => {
        void compute(mount, tok);
    });
}

async function compute(mount, tok) {
    hideErr();
    const err = validateInputs(state.mode, state.params);
    if (err) { showErr(err); return; }

    // Local sanity preview so the user sees instant numbers even if the
    // network round-trip stalls. Backend response overwrites this on
    // success.
    if (state.mode === 'decompose') {
        const d = decomposeLocal(state.params.sigma_25_call, state.params.sigma_25_put, state.params.sigma_atm);
        renderDecomposeSummary(d, /*fromBackend=*/false);
    } else {
        const r = reconstructLocal(state.params.atm, state.params.rr, state.params.bf);
        renderReconstructSummary(r, /*fromBackend=*/false);
    }

    let res;
    try {
        res = await api.anlyRiskReversalBfCalc(buildBody(state.mode, state.params));
        if (!res) throw new Error(t('view.rr_butterfly.error.null'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    if (state.mode === 'decompose') {
        renderDecomposeSummary({
            atm: res.atm,
            rr: res.risk_reversal,
            bf: res.butterfly,
            skew_zscore: res.skew_zscore,
        }, /*fromBackend=*/true);
    } else {
        renderReconstructSummary({
            sigma_25_call: res.sigma_25_call,
            sigma_25_put: res.sigma_25_put,
        }, /*fromBackend=*/true);
    }
}

function renderDecomposeSummary(d, fromBackend) {
    const rrClass = (d.rr ?? 0) >= 0 ? 'pos' : 'neg';
    const bfClass = (d.bf ?? 0) >= 0 ? 'pos' : 'neg';
    const skewClass = (d.skew_zscore ?? 0) >= 0 ? 'pos' : 'neg';
    document.getElementById('rr-summary').innerHTML = [
        card(t('view.rr_butterfly.card.atm'),                fmtVolPct(d.atm), '', subtitle(fromBackend)),
        card(t('view.rr_butterfly.card.risk_reversal_rr'), fmtVolPct(d.rr),   rrClass,
            `<div class="vc-row"><span class="muted" data-i18n="view.rr_butterfly.row.sigma_diff">σ_25C − σ_25P</span> <strong>${fmtVolPct(d.rr)}</strong></div>
             <div class="vc-row"><span class="muted">interp</span> <strong>${rrInterp(d.rr)}</strong></div>`),
        card(t('view.rr_butterfly.card.butterfly_bf'),     fmtVolPct(d.bf),   bfClass,
            `<div class="vc-row"><span class="muted">(σ_25C + σ_25P)/2 − σ_ATM</span> <strong>${fmtVolPct(d.bf)}</strong></div>
             <div class="vc-row"><span class="muted">interp</span> <strong>${bfInterp(d.bf)}</strong></div>`),
        card(t('view.rr_butterfly.card.skew_z_score'),       fmtSkewZ(d.skew_zscore), skewClass,
            `<div class="vc-row"><span class="muted">RR / ATM</span> <strong>${fmtSkewZ(d.skew_zscore)}</strong></div>`),
    ].join('');
}

function renderReconstructSummary(r, fromBackend) {
    document.getElementById('rr-summary').innerHTML = [
        card(t('view.rr_butterfly.card.25c_call_wing'), fmtVolPct(r.sigma_25_call), '', subtitle(fromBackend)),
        card(t('view.rr_butterfly.card.25p_put_wing'),  fmtVolPct(r.sigma_25_put),  '', subtitle(fromBackend)),
        card(t('view.rr_butterfly.card.spread'),            fmtVolPct(r.sigma_25_call - r.sigma_25_put), '',
            `<div class="vc-row"><span class="muted" data-i18n="view.rr_butterfly.row.sigma_diff">σ_25C − σ_25P</span>
                <strong>${fmtVolPct(r.sigma_25_call - r.sigma_25_put)}</strong></div>`),
    ].join('');
}

function card(label, value, valueCls, body = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${valueCls}">${esc(value)}</div>
        ${body ? `<div class="value rr-summary-value">${body}</div>` : ''}
    </div>`;
}

function subtitle(fromBackend) {
    return `<div class="vc-row"><span class="muted" data-i18n="view.rr_butterfly.row.source">source</span>
        <strong>${esc(t(fromBackend ? 'view.rr_butterfly.source.backend' : 'view.rr_butterfly.source.local'))}</strong></div>`;
}

function rrInterp(rr) {
    if (!Number.isFinite(rr)) return '—';
    if (Math.abs(rr) < 0.0005) return t('view.rr_butterfly.rr.flat');
    return t(rr > 0 ? 'view.rr_butterfly.rr.calls_richer' : 'view.rr_butterfly.rr.puts_richer');
}

function bfInterp(bf) {
    if (!Number.isFinite(bf)) return '—';
    if (Math.abs(bf) < 0.0005) return t('view.rr_butterfly.bf.flat');
    return t(bf > 0 ? 'view.rr_butterfly.bf.wings_rich' : 'view.rr_butterfly.bf.wings_cheap');
}

function showErr(msg) {
    const el = document.getElementById('rr-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('rr-err').style.display = 'none'; }
