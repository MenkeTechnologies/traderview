// Cohort-tilt view — TopstepX-style "The Tilt" indicator. Aggregates a
// cohort of traders' positions and surfaces the most lopsided symbols
// with 5-tier bias classification.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePositionBlob, validateInputs, buildBody, localAggregate,
    biasBadge, cohortLongRatio, makeDemoPositions,
    fmtPct, fmtSignedInt, symbolColor,
} from '../_cohort_tilt_inputs.js';

let state = { positions: makeDemoPositions('mixed') };

export async function renderCohortTilt(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// COHORT TILT</h1>

        <div class="chart-panel">
            <h2>Cohort positions <small class="muted">(per line: <code>trader_id SYMBOL net_contracts</code> — signed: + long, - short, 0 flat)</small></h2>
            <textarea id="ct-blob" rows="10" placeholder="L0 ES 3&#10;S0 ES -3&#10;a NQ 1">${esc(positionsToBlob(state.positions))}</textarea>
            <div class="inline-form">
                <button id="ct-run" class="primary" type="button">Aggregate</button>
                <button id="ct-demo-mixed"  class="secondary" type="button">Demo: 4 symbols mixed bias</button>
                <button id="ct-demo-long"   class="secondary" type="button">Demo: strongly long ES</button>
                <button id="ct-demo-short"  class="secondary" type="button">Demo: strongly short NQ</button>
                <button id="ct-demo-flat"   class="secondary" type="button">Demo: all flat (no active)</button>
                <button id="ct-demo-cross"  class="secondary" type="button">Demo: same trader cross-symbol</button>
            </div>
            <p class="muted">Bias buckets: ≥75% strongly long, ≥60% long, 40–60% balanced, ≥25% short, &lt;25% strongly short. Symbols sorted by lopsidedness |long_ratio − 0.5| desc.</p>
        </div>

        <div id="ct-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Long-ratio bars (0% = all short, 50% = balanced, 100% = all long)</h2>
            <div id="ct-bars"></div>
            <p class="muted">Bar fill colored by bias. Track at 50% midline. Symbols with all-flat traders show "—".</p>
        </div>

        <div class="chart-panel">
            <h2>Per-symbol detail</h2>
            <div id="ct-table"></div>
        </div>

        <div id="ct-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.positions = makeDemoPositions(kind);
        document.getElementById('ct-blob').value = positionsToBlob(state.positions);
    };
    document.getElementById('ct-demo-mixed').addEventListener('click', () => loadDemo('mixed'));
    document.getElementById('ct-demo-long').addEventListener('click',  () => loadDemo('strongly-long'));
    document.getElementById('ct-demo-short').addEventListener('click', () => loadDemo('strongly-short'));
    document.getElementById('ct-demo-flat').addEventListener('click',  () => loadDemo('all-flat'));
    document.getElementById('ct-demo-cross').addEventListener('click', () => loadDemo('cross-symbol'));
    document.getElementById('ct-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function positionsToBlob(positions) {
    return positions.map(p => `${p.trader_id} ${p.symbol} ${p.net_contracts}`).join('\n');
}

function readInputs() {
    const parsed = parsePositionBlob(document.getElementById('ct-blob').value);
    if (parsed.errors.length) {
        showErr(`Parse errors: ${parsed.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; ')}`);
        return;
    }
    hideErr();
    state.positions = parsed.positions;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.positions);
    if (err) { showErr(err); return; }
    const local = localAggregate(state.positions);
    renderSummary(local, true);
    renderBars(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.cohortTilt(buildBody(state.positions));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderBars(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localAggregate(state.positions);
    const parity = reportEq(report, local);
    const cohortR = cohortLongRatio(report);
    const cohortBias = cohortR == null ? null : biasBadge(localClassify(cohortR));
    const top = report.by_symbol[0] || null;
    const topBadge = top ? biasBadge(top.bias) : null;
    document.getElementById('ct-summary').innerHTML = [
        card('Active traders', String(report.active_traders) + (pending ? ' (local)' : ''),
            report.active_traders > 0 ? 'pos' : 'neg'),
        card('Symbols tracked', String(report.by_symbol.length)),
        card('Most lopsided',  report.most_lopsided_symbol || '—',
            top && top.long_ratio != null ? topBadge.cls : ''),
        card('Top bias',       topBadge ? topBadge.label : '—',
            topBadge ? topBadge.cls : ''),
        card('Top long ratio', top ? fmtPct(top.long_ratio) : '—',
            topBadge ? topBadge.cls : ''),
        card('Cohort long ratio', cohortR == null ? '—' : fmtPct(cohortR),
            cohortBias ? cohortBias.cls : ''),
        card('Cohort bias',    cohortBias ? cohortBias.label : '—',
            cohortBias ? cohortBias.cls : ''),
        card('Local parity',   parity ? 'OK' : 'DIVERGED', parity ? 'pos' : 'neg'),
    ].join('');
}

function localClassify(r) {
    if (r >= 0.75) return 'strongly_long';
    if (r >= 0.60) return 'long';
    if (r >= 0.40) return 'balanced';
    if (r >= 0.25) return 'short';
    return 'strongly_short';
}

function reportEq(a, b) {
    if (!a || !b) return false;
    if (a.active_traders !== b.active_traders) return false;
    if (a.most_lopsided_symbol !== b.most_lopsided_symbol) return false;
    if (a.by_symbol.length !== b.by_symbol.length) return false;
    for (let i = 0; i < a.by_symbol.length; i++) {
        const x = a.by_symbol[i], y = b.by_symbol[i];
        if (x.symbol !== y.symbol) return false;
        if (x.long_traders !== y.long_traders) return false;
        if (x.short_traders !== y.short_traders) return false;
        if (x.bias !== y.bias) return false;
    }
    return true;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderBars(report) {
    const wrap = document.getElementById('ct-bars');
    if (!report.by_symbol.length) { wrap.innerHTML = '<div class="muted">No symbols.</div>'; return; }
    // Track + fill. Use semantic CSS classes already defined for daily-loss-limit.
    wrap.innerHTML = report.by_symbol.map((s, i) => {
        const ratio = s.long_ratio;
        const pct = ratio == null ? 50 : ratio * 100;
        const badge = biasBadge(s.bias);
        const color = symbolColor(i);
        return `
            <div class="dl-bar-row" style="margin-bottom:8px">
                <div style="display:flex;justify-content:space-between;font-size:12px;margin-bottom:4px">
                    <span style="color:${esc(color)};font-weight:bold">●</span>
                    <strong>${esc(s.symbol)}</strong>
                    <span class="${badge.cls}">${esc(badge.label)}</span>
                    <span class="muted">${ratio == null ? '—' : esc(fmtPct(ratio))}</span>
                    <span class="muted">${s.long_traders}L / ${s.short_traders}S / ${s.flat_traders}F</span>
                    <span class="${s.net_contracts >= 0 ? 'pos' : 'neg'}">${esc(fmtSignedInt(s.net_contracts))} ct</span>
                </div>
                <div class="dl-bar-track">
                    <div class="dl-bar-fill ${badge.cls === 'pos' ? 'dl-fill-ok' : badge.cls === 'neg' ? 'dl-fill-kill' : 'dl-fill-warn'}" data-pct="${pct.toFixed(2)}"></div>
                    <div class="dl-bar-mark dl-mark-cut" data-pct="50"></div>
                </div>
            </div>
        `;
    }).join('');
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.dl-bar-fill').forEach(el => { el.style.width = el.dataset.pct + '%'; });
        wrap.querySelectorAll('.dl-bar-mark').forEach(el => { el.style.left  = el.dataset.pct + '%'; });
    });
}

function renderTable(report) {
    const wrap = document.getElementById('ct-table');
    if (!report.by_symbol.length) { wrap.innerHTML = '<div class="muted">No symbols.</div>'; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th>Symbol</th><th>Bias</th>
                <th>Long</th><th>Short</th><th>Flat</th>
                <th>Long ratio</th><th>Net contracts</th><th>Lopsidedness</th>
            </tr></thead>
            <tbody>
                ${report.by_symbol.map((s, i) => {
                    const badge = biasBadge(s.bias);
                    const lop = s.long_ratio == null ? 0 : Math.abs(s.long_ratio - 0.5);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td><span style="color:${esc(symbolColor(i))};font-weight:bold">●</span> <strong>${esc(s.symbol)}</strong></td>
                        <td class="${badge.cls}">${esc(badge.label)}</td>
                        <td class="pos">${s.long_traders}</td>
                        <td class="neg">${s.short_traders}</td>
                        <td class="muted">${s.flat_traders}</td>
                        <td class="${badge.cls}">${esc(s.long_ratio == null ? '—' : fmtPct(s.long_ratio))}</td>
                        <td class="${s.net_contracts >= 0 ? 'pos' : 'neg'}">${esc(fmtSignedInt(s.net_contracts))}</td>
                        <td>${esc(fmtPct(lop))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('ct-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ct-err').style.display = 'none'; }
