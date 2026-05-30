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

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { pivotText: '', tolerance: 0.05 };

export async function renderCypherPattern(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cypher_pattern.h1.cypher_harmonic_pattern" class="view-title">// CYPHER HARMONIC PATTERN</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.cypher_pattern.h2.pivots">Pivots</h2>
            <p class="muted" data-i18n-html="view.cypher_pattern.help">Paste <code>index price H|L</code> per line. Pivots must
                alternate high/low. Demo loads a 5-pivot bullish Cypher with tolerance 0.10
                (the more permissive setting most retail screeners use).</p>
            <textarea id="cy-pivots" rows="6" placeholder="0 100 L&#10;10 130 H&#10;20 115 L&#10;30 134.08 H&#10;40 106.42 L" data-tip="view.cypher_pattern.tip.pivots"></textarea>
            <div class="inline-form">
                <label><span data-i18n="view.cypher_pattern.label.tolerance">Tolerance (typical 0.03-0.10)</span>
                    <input id="cy-tol" type="number" step="any" min="0" max="0.5" value="${state.tolerance}" data-tip="view.cypher_pattern.tip.tolerance"></label>
                <button data-i18n="view.cypher_pattern.btn.load_demo_5_pivot_bullish_cypher_tol_0_10" id="cy-demo" class="secondary" type="button" data-tip="view.cypher_pattern.tip.demo" data-shortcut="cypher_pattern_demo">Load demo (5-pivot bullish Cypher, tol=0.10)</button>
                <button data-i18n="view.cypher_pattern.btn.clear" id="cy-clear" class="secondary" type="button" data-tip="view.cypher_pattern.tip.clear">Clear</button>
                <button data-i18n="view.cypher_pattern.btn.detect" id="cy-run" class="primary" type="button" data-tip="view.cypher_pattern.tip.run" data-shortcut="cypher_pattern_run">Detect</button>
            </div>
        </div>

        <div id="cy-errors" class="boot" style="display:none"></div>
        <div id="cy-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cypher_pattern.h2.pivot_chart_with_xabcd_overlay">Pivot chart with XABCD overlay</h2>
            <div id="cy-chart" style="height:320px"></div>
            <p data-i18n="view.cypher_pattern.hint.cyan_pivot_price_connecting_line_yellow_dots_pivot" class="muted">Cyan = pivot price connecting line. Yellow dots = pivots. When
                a Cypher matches, X/A/B/C/D markers are highlighted on the first match's pivots.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cypher_pattern.h2.quality_chart">Per-match geometric quality (lower = closer to ideal Cypher)</h2>
            <div id="cy-quality-chart" style="height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cypher_pattern.h2.pattern_matches">Pattern matches</h2>
            <div id="cy-matches"></div>
        </div>

        <div id="cy-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('cy-demo').addEventListener('click', () => {
        const p = makeDemoPivots();
        document.getElementById('cy-pivots').value =
            p.map(x => `${x.index} ${x.price} ${x.is_high ? 'H' : 'L'}`).join('\n');
        document.getElementById('cy-tol').value = DEMO_TOLERANCE;
        showToast(t('view.cypher_pattern.toast.demo_loaded', { n: p.length }), { level: 'info' });
    });
    document.getElementById('cy-clear').addEventListener('click', () => {
        document.getElementById('cy-pivots').value = '';
        showToast(t('view.cypher_pattern.toast.cleared'), { level: 'info' });
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
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        showToast(t('view.cypher_pattern.toast.parse_error', { n: errors.length }), { level: 'warning' });
        if (pivots.length === 0) return;
    }
    const err = validateInputs(pivots, state.tolerance);
    if (err) { showErr(err); showToast(t('view.cypher_pattern.toast.invalid'), { level: 'warning' }); return; }
    let matches;
    try {
        matches = await api.anlyCypherPattern(buildBody(pivots, state.tolerance));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.cypher_pattern.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(matches || [], pivots);
    renderChart(pivots, matches || []);
    renderQualityChart(matches || []);
    renderMatches(matches || []);
    const n = (matches || []).length;
    showToast(t('view.cypher_pattern.toast.detected', { n, pivots: pivots.length }), { level: n > 0 ? 'success' : 'info' });
}

function renderSummary(matches, pivots) {
    const bull = matches.filter(m => m.direction === 'bullish').length;
    const bear = matches.filter(m => m.direction === 'bearish').length;
    const best = matches.reduce((b, m) => {
        const q = patternQuality(m);
        return (b == null || q > b.q) ? { match: m, q } : b;
    }, null);
    document.getElementById('cy-summary').innerHTML = [
        card(t('view.cypher_pattern.card.pivots'),         String(pivots.length)),
        card(t('view.cypher_pattern.card.matches'),        String(matches.length)),
        card(t('view.cypher_pattern.card.bullish'),        String(bull), bull ? 'pos' : ''),
        card(t('view.cypher_pattern.card.bearish'),        String(bear), bear ? 'neg' : ''),
        card(t('view.cypher_pattern.card.best_quality'),   best ? qualityGrade(best.q).label + ' · ' + fmtN(best.q, 3) : '—',
            best ? qualityGrade(best.q).cls : ''),
        card(t('view.cypher_pattern.card.tolerance_used'), String(state.tolerance)),
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
        el.innerHTML = `<div class="muted" data-i18n="view.cypher_pattern.empty.pivots">No pivots.</div>`;
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
            { label: t('chart.series.pivot_index') },
            { label: t('chart.series.pivots'), stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: true, size: 8, stroke: '#ffd84a', fill: '#ffd84a' } },
            { label: t('chart.series.match_xabcd'), stroke: '#39ff14', width: 0,
              points: { show: true, size: 14, stroke: '#39ff14', fill: 'transparent' } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
        legend: { show: true },
    }, [xs, ys, matchMarkers], el);
}

function renderQualityChart(matches) {
    if (!window.uPlot) return;
    const el = document.getElementById('cy-quality-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!matches.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.cypher_pattern.empty_quality_chart">${esc(t('view.cypher_pattern.empty_quality_chart'))}</div>`;
        return;
    }
    const qs = matches.map(m => patternQuality(m));
    const xs = matches.map((_, i) => i + 1);
    const labels = matches.map((m, i) => `${i + 1}·${m.direction === 'bullish' ? '↑' : '↓'}`);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.cypher_pattern.chart.match_idx') },
            { label: t('view.cypher_pattern.chart.quality'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, qs], el);
}

function renderMatches(matches) {
    const wrap = document.getElementById('cy-matches');
    if (!matches.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.cypher_pattern.empty.matches">No Cypher patterns matched at current tolerance.</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.cypher_pattern.th.direction">Direction</th><th data-i18n="view.cypher_pattern.th.grade">Grade</th>
                <th data-i18n="view.cypher_pattern.th.x_idx">X idx</th><th data-i18n="view.cypher_pattern.th.a_idx">A idx</th><th data-i18n="view.cypher_pattern.th.b_idx">B idx</th><th data-i18n="view.cypher_pattern.th.c_idx">C idx</th><th data-i18n="view.cypher_pattern.th.d_idx">D idx</th>
                <th data-i18n="view.cypher_pattern.th.ab_xa">AB/XA</th><th data-i18n="view.cypher_pattern.th.bc_ab">BC/AB</th><th data-i18n="view.cypher_pattern.th.cd_xc">CD/XC</th><th data-i18n="view.cypher_pattern.th.ad_xa">AD/XA</th>
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
