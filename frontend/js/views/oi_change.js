// OI Change view — open-interest surge / unwind alerter across an options chain.
//
// "Where is institutional positioning building today?" Compares each
// strike's current OI to its rolling baseline; emits an alert when
// pct_change exceeds threshold AND current OI ≥ min_oi (suppresses
// micro-strike noise). Surge on the call side = upside positioning;
// surge on the put side = downside hedge build.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSnapshotBlob, validateInputs, buildBody,
    alertTier, flowDirection, summarize,
    makeDemoSnapshots,
    fmtN, fmtInt, fmtPct, fmtSignedInt,
} from '../_oi_change_inputs.js';

let state = { snapshotsText: '', pctThreshold: 0.25, minOi: 1000 };

export async function renderOiChange(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// OI CHANGE · POSITIONING SURGE ALERTER</h1>

        <div class="chart-panel">
            <h2>Strike-level OI snapshots</h2>
            <p class="muted">One line per strike: <code>strike call_oi put_oi call_baseline put_baseline</code>.
                Baseline = trailing 20-day average (or your own reference). Demo loads
                an 8-strike chain with engineered surges on the 510 call and 470 put.</p>
            <textarea id="oi-snap" rows="8" placeholder="500 25000 6000 24000 6200&#10;510 32000 3000 12000 3100&#10;..."></textarea>
            <div class="inline-form">
                <label>Pct threshold (e.g. 0.25 = 25%)
                    <input id="oi-pct" type="number" step="any" min="0" value="${state.pctThreshold}"></label>
                <label>Min OI (suppress micro-strike noise)
                    <input id="oi-min" type="number" step="1" min="0" value="${state.minOi}"></label>
                <button id="oi-demo" class="secondary" type="button">Load demo (8 strikes, surge call+put)</button>
                <button id="oi-clear" class="secondary" type="button">Clear</button>
                <button id="oi-run" class="primary" type="button">Analyze</button>
            </div>
        </div>

        <div id="oi-errors" class="boot" style="display:none"></div>
        <div id="oi-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Call-side OI alerts</h2>
            <div id="oi-calls"></div>
            <p class="muted">Sorted biggest absolute change first. SURGE/STRONG rows
                are where upside positioning is concentrating today.</p>
        </div>

        <div class="chart-panel">
            <h2>Put-side OI alerts</h2>
            <div id="oi-puts"></div>
            <p class="muted">Sorted biggest absolute change first. SURGE on puts =
                institutional hedging or directional bearish bets.</p>
        </div>

        <div id="oi-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('oi-demo').addEventListener('click', () => {
        const snaps = makeDemoSnapshots();
        document.getElementById('oi-snap').value =
            snaps.map(s => `${s.strike} ${s.call_oi} ${s.put_oi} ${s.call_oi_baseline} ${s.put_oi_baseline}`).join('\n');
    });
    document.getElementById('oi-clear').addEventListener('click', () => {
        document.getElementById('oi-snap').value = '';
    });
    document.getElementById('oi-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.snapshotsText = document.getElementById('oi-snap').value;
    state.pctThreshold = Number(document.getElementById('oi-pct').value);
    state.minOi = parseInt(document.getElementById('oi-min').value, 10);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('oi-errors');
    errs.style.display = 'none';
    const { snapshots, errors } = parseSnapshotBlob(state.snapshotsText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (snapshots.length === 0) return;
    }
    const err = validateInputs(snapshots, state.pctThreshold, state.minOi);
    if (err) { showErr(err); return; }
    let report;
    try {
        report = await api.optCalcOiChange(buildBody(snapshots, state.pctThreshold, state.minOi));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report, snapshots);
    renderTable('oi-calls', report.call_alerts || []);
    renderTable('oi-puts',  report.put_alerts  || []);
}

function renderSummary(report, snapshots) {
    const s = summarize(report);
    const callPutSkew = s.netCallChange - s.netPutChange;
    document.getElementById('oi-summary').innerHTML = [
        card('Strikes scanned',  String(snapshots.length)),
        card('Call alerts',      String(s.totalCallAlerts), s.totalCallAlerts ? 'pos' : ''),
        card('Put alerts',       String(s.totalPutAlerts),  s.totalPutAlerts ? 'neg' : ''),
        card('Net call ΔOI',     fmtSignedInt(s.netCallChange), s.netCallChange >= 0 ? 'pos' : 'neg'),
        card('Net put ΔOI',      fmtSignedInt(s.netPutChange),  s.netPutChange >= 0 ? 'neg' : 'pos'),
        card('Hot call strike',  s.maxCallStrike != null ? fmtN(s.maxCallStrike) : '—', 'pos'),
        card('Hot put strike',   s.maxPutStrike  != null ? fmtN(s.maxPutStrike)  : '—', 'neg'),
        card('Net positioning',  fmtSignedInt(callPutSkew),
            callPutSkew > 0 ? 'pos' : callPutSkew < 0 ? 'neg' : ''),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTable(elId, alerts) {
    const wrap = document.getElementById(elId);
    if (!alerts.length) {
        wrap.innerHTML = '<div class="muted">No alerts.</div>';
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>Strike</th><th>Tier</th><th>Flow</th>
                <th>Current OI</th><th>Baseline</th>
                <th>Δ OI</th><th>Δ %</th>
            </tr></thead>
            <tbody>
                ${alerts.map(a => {
                    const tier = alertTier(a);
                    const flow = flowDirection(a.abs_change);
                    return `<tr>
                        <td>${esc(fmtN(a.strike))}</td>
                        <td class="${tier.cls}">${esc(tier.label)}</td>
                        <td class="${flow.cls}">${esc(flow.label)}</td>
                        <td>${esc(fmtInt(a.current_oi))}</td>
                        <td>${esc(fmtInt(a.baseline_oi))}</td>
                        <td class="${a.abs_change >= 0 ? 'neg' : 'pos'}">${esc(fmtSignedInt(a.abs_change))}</td>
                        <td class="${a.pct_change >= 0 ? 'neg' : 'pos'}">${esc(fmtPct(a.pct_change))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('oi-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('oi-err').style.display = 'none'; }
