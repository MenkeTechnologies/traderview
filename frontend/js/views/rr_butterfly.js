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

const DEFAULTS = {
    decompose:   { sigma_25_call: 0.085, sigma_25_put: 0.097, sigma_atm: 0.090 },
    reconstruct: { atm: 0.090, rr: -0.012, bf: 0.001 },
};

let state = { mode: 'decompose', params: { ...DEFAULTS.decompose } };

export async function renderRrButterfly(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 class="view-title">// RR / BF CALCULATOR</h1>

        <div class="chart-panel">
            <h2>Mode</h2>
            <div class="inline-form">
                <label>Direction
                    <select id="rr-mode">
                        <option value="decompose"   ${state.mode === 'decompose'   ? 'selected' : ''}>Decompose → ATM + RR + BF</option>
                        <option value="reconstruct" ${state.mode === 'reconstruct' ? 'selected' : ''}>Reconstruct → σ wings</option>
                    </select></label>
                <button id="rr-run" class="primary" type="button">Compute</button>
            </div>
            <p class="muted">
                FX vol quotes are conventionally in vol-points (percent). RR &gt; 0 = calls
                priced richer than puts (right tail fear); BF &gt; 0 = wings priced richer
                than ATM (smile curvature premium).
            </p>
        </div>

        <div class="chart-panel">
            <h2>Inputs</h2>
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
            <label>σ_25C (call wing)
                <input id="rr-sc"  type="number" step="any" min="0" value="${state.params.sigma_25_call}"></label>
            <label>σ_25P (put wing)
                <input id="rr-sp"  type="number" step="any" min="0" value="${state.params.sigma_25_put}"></label>
            <label>σ_ATM
                <input id="rr-atm" type="number" step="any" min="0" value="${state.params.sigma_atm}"></label>
        `;
        wrap.querySelector('#rr-sc').addEventListener('change',  e => state.params.sigma_25_call = Number(e.target.value));
        wrap.querySelector('#rr-sp').addEventListener('change',  e => state.params.sigma_25_put  = Number(e.target.value));
        wrap.querySelector('#rr-atm').addEventListener('change', e => state.params.sigma_atm     = Number(e.target.value));
    } else {
        wrap.innerHTML = `
            <label>ATM IV
                <input id="rr-atm-in" type="number" step="any" min="0" value="${state.params.atm}"></label>
            <label>Risk reversal (RR)
                <input id="rr-rr"     type="number" step="any" value="${state.params.rr}"></label>
            <label>Butterfly (BF)
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
        if (!res) throw new Error('calculator returned null (input out of domain)');
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
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
        card('ATM',                fmtVolPct(d.atm), '', subtitle(fromBackend)),
        card('Risk Reversal (RR)', fmtVolPct(d.rr),   rrClass,
            `<div class="vc-row"><span class="muted">σ_25C − σ_25P</span> <strong>${fmtVolPct(d.rr)}</strong></div>
             <div class="vc-row"><span class="muted">interp</span> <strong>${rrInterp(d.rr)}</strong></div>`),
        card('Butterfly (BF)',     fmtVolPct(d.bf),   bfClass,
            `<div class="vc-row"><span class="muted">(σ_25C + σ_25P)/2 − σ_ATM</span> <strong>${fmtVolPct(d.bf)}</strong></div>
             <div class="vc-row"><span class="muted">interp</span> <strong>${bfInterp(d.bf)}</strong></div>`),
        card('Skew z-score',       fmtSkewZ(d.skew_zscore), skewClass,
            `<div class="vc-row"><span class="muted">RR / ATM</span> <strong>${fmtSkewZ(d.skew_zscore)}</strong></div>`),
    ].join('');
}

function renderReconstructSummary(r, fromBackend) {
    document.getElementById('rr-summary').innerHTML = [
        card('σ_25C (call wing)', fmtVolPct(r.sigma_25_call), '', subtitle(fromBackend)),
        card('σ_25P (put wing)',  fmtVolPct(r.sigma_25_put),  '', subtitle(fromBackend)),
        card('Spread',            fmtVolPct(r.sigma_25_call - r.sigma_25_put), '',
            `<div class="vc-row"><span class="muted">σ_25C − σ_25P</span>
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
    return `<div class="vc-row"><span class="muted">source</span>
        <strong>${fromBackend ? 'backend' : 'local preview…'}</strong></div>`;
}

function rrInterp(rr) {
    if (!Number.isFinite(rr)) return '—';
    if (Math.abs(rr) < 0.0005) return 'flat skew';
    return rr > 0 ? 'calls richer (right-tail bid)' : 'puts richer (left-tail bid)';
}

function bfInterp(bf) {
    if (!Number.isFinite(bf)) return '—';
    if (Math.abs(bf) < 0.0005) return 'flat smile';
    return bf > 0 ? 'wings rich vs ATM (kurtosis bid)' : 'wings cheap vs ATM';
}

function showErr(msg) {
    const el = document.getElementById('rr-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('rr-err').style.display = 'none'; }
