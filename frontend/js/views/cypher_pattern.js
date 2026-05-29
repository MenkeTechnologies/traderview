// Cypher Pattern view — Darren Oglesbee XABCD harmonic detector.
//
// Cypher's signature is the BC leg overshooting A (the only harmonic
// variant where C extends past A). Ratio constraints:
//   AB / XA      ∈ [0.382, 0.618]
//   BC / AB      ∈ [1.130, 1.414]   (extension past A)
//   CD / XC      ∈ [1.272, 2.000]   (measured from X to C, not B to C)
//   AD / XA      ≈ 0.786            (key Cypher constraint)

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePivotBlob, validateInputs, buildBody,
    dirBadge, patternQuality, qualityGrade,
    makeDemoPivots, DEMO_TOLERANCE,
    fmtN, fmtRatio,
} from '../_cypher_pattern_inputs.js';

let state = { pivotText: '', tolerance: 0.05 };

export async function renderCypherPattern(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// CYPHER HARMONIC PATTERN</h1>

        <div class="chart-panel">
            <h2>Pivots</h2>
            <p class="muted">Paste <code>index price H|L</code> per line. Pivots must
                alternate high/low. Demo loads a 5-pivot bullish Cypher with tolerance 0.10
                (the more permissive setting most retail screeners use).</p>
            <textarea id="cy-pivots" rows="6" placeholder="0 100 L&#10;10 130 H&#10;20 115 L&#10;30 134.08 H&#10;40 106.42 L"></textarea>
            <div class="inline-form">
                <label>Tolerance (typical 0.03-0.10)
                    <input id="cy-tol" type="number" step="any" min="0" max="0.5" value="${state.tolerance}"></label>
                <button id="cy-demo" class="secondary" type="button">Load demo (5-pivot bullish Cypher, tol=0.10)</button>
                <button id="cy-clear" class="secondary" type="button">Clear</button>
                <button id="cy-run" class="primary" type="button">Detect</button>
            </div>
        </div>

        <div id="cy-errors" class="boot" style="display:none"></div>
        <div id="cy-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Pivot chart with XABCD overlay</h2>
            <div id="cy-chart" style="height:320px"></div>
            <p class="muted">Cyan = pivot price connecting line. Yellow dots = pivots. When
                a Cypher matches, X/A/B/C/D markers are highlighted on the first match's pivots.</p>
        </div>

        <div class="chart-panel">
            <h2>Pattern matches</h2>
            <div id="cy-matches"></div>
        </div>

        <div id="cy-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('cy-demo').addEventListener('click', () => {
        const p = makeDemoPivots();
        document.getElementById('cy-pivots').value =
            p.map(x => `${x.index} ${x.price} ${x.is_high ? 'H' : 'L'}`).join('\n');
        document.getElementById('cy-tol').value = DEMO_TOLERANCE;
    });
    document.getElementById('cy-clear').addEventListener('click', () => {
        document.getElementById('cy-pivots').value = '';
    });
    document.getElementById('cy-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.pivotText = document.getElementById('cy-pivots').value;
    state.tolerance = Number(document.getElementById('cy-tol').value);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('cy-errors');
    errs.style.display = 'none';
    const { pivots, errors } = parsePivotBlob(state.pivotText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (pivots.length === 0) return;
    }
    const err = validateInputs(pivots, state.tolerance);
    if (err) { showErr(err); return; }
    let matches;
    try {
        matches = await api.anlyCypherPattern(buildBody(pivots, state.tolerance));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(matches || [], pivots);
    renderChart(pivots, matches || []);
    renderMatches(matches || []);
}

function renderSummary(matches, pivots) {
    const bull = matches.filter(m => m.direction === 'bullish').length;
    const bear = matches.filter(m => m.direction === 'bearish').length;
    const best = matches.reduce((b, m) => {
        const q = patternQuality(m);
        return (b == null || q > b.q) ? { match: m, q } : b;
    }, null);
    document.getElementById('cy-summary').innerHTML = [
        card('Pivots',         String(pivots.length)),
        card('Matches',        String(matches.length)),
        card('Bullish',        String(bull), bull ? 'pos' : ''),
        card('Bearish',        String(bear), bear ? 'neg' : ''),
        card('Best quality',   best ? qualityGrade(best.q).label + ' · ' + fmtN(best.q, 3) : '—',
            best ? qualityGrade(best.q).cls : ''),
        card('Tolerance used', String(state.tolerance)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(pivots, matches) {
    if (!window.uPlot) return;
    const el = document.getElementById('cy-chart');
    if (!pivots.length) {
        el.innerHTML = '<div class="muted">No pivots.</div>';
        return;
    }
    const sorted = [...pivots].sort((a, b) => a.index - b.index);
    const xs = sorted.map(p => p.index);
    const ys = sorted.map(p => p.price);
    // Highlight the first match's 5 pivots on a separate marker series.
    const firstMatch = matches[0];
    const matchMarkers = sorted.map(p => {
        if (!firstMatch) return null;
        const isMatchPivot = [firstMatch.x, firstMatch.a, firstMatch.b, firstMatch.c, firstMatch.d]
            .some(mp => mp && mp.index === p.index);
        return isMatchPivot ? p.price : null;
    });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: 'pivot index' },
            { label: 'pivots', stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: true, size: 8, stroke: '#ffd84a', fill: '#ffd84a' } },
            { label: 'match XABCD', stroke: '#39ff14', width: 0,
              points: { show: true, size: 14, stroke: '#39ff14', fill: 'transparent' } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, ys, matchMarkers], el);
}

function renderMatches(matches) {
    const wrap = document.getElementById('cy-matches');
    if (!matches.length) {
        wrap.innerHTML = '<div class="muted">No Cypher patterns matched at current tolerance.</div>';
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th>Direction</th><th>Grade</th>
                <th>X idx</th><th>A idx</th><th>B idx</th><th>C idx</th><th>D idx</th>
                <th>AB/XA</th><th>BC/AB</th><th>CD/XC</th><th>AD/XA</th>
            </tr></thead>
            <tbody>
                ${matches.map((m, i) => {
                    const d = dirBadge(m.direction);
                    const g = qualityGrade(patternQuality(m));
                    return `<tr>
                        <td>${i + 1}</td>
                        <td class="${d.cls}">${esc(d.label)}</td>
                        <td class="${g.cls}">${esc(g.label)}</td>
                        <td>${m.x.index}</td>
                        <td>${m.a.index}</td>
                        <td>${m.b.index}</td>
                        <td>${m.c.index}</td>
                        <td>${m.d.index}</td>
                        <td>${esc(fmtRatio(m.ab_ratio))}</td>
                        <td>${esc(fmtRatio(m.bc_ratio))}</td>
                        <td>${esc(fmtRatio(m.cd_to_xc_ratio))}</td>
                        <td>${esc(fmtRatio(m.ad_ratio))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('cy-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cy-err').style.display = 'none'; }
