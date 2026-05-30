// Murrey Math Levels view — T. Henning Murrey octave-grid S/R.
//
// Divides the lookback range into eighths (the "octave") plus 4
// extension levels for breakout targets. Each level carries traditional
// significance — 0/8 ultimate support, 4/8 mid pivot, 8/8 ultimate
// resistance, 2/8 and 6/8 reversal pivots, etc.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarBlob, validateInputs, buildBody,
    significanceOf, pricePosition, pricePositionLabel, bracketingLevels,
    makeDemoBars, fmtN, fmtPct,
} from '../_murrey_math_inputs.js';

import { t } from '../i18n.js';
let state = { barText: '', lookback: 64 };

export async function renderMurreyMath(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.murrey_math.h1.murrey_math_levels" class="view-title">// MURREY MATH LEVELS</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.murrey_math.h2.hlc_bars">HLC bars</h2>
            <p class="muted" data-i18n-html="view.murrey_math.help">Paste <code>high low close</code> per line. Murrey
                auto-detects the octave by rounding the lookback range to a
                power-of-2 base. Demo loads 80 bars in a ~10-point oscillating range.</p>
            <textarea id="mm-bars" rows="6" placeholder="100.6 99.4 100.0&#10;100.8 99.6 100.2&#10;..."></textarea>
            <div class="inline-form">
                <label><span data-i18n="view.murrey_math.label.lookback">Lookback bars</span>
                    <input id="mm-lb" type="number" step="1" min="1" value="${state.lookback}"></label>
                <button data-i18n="view.murrey_math.btn.load_demo_80_bars_10_pt_range" id="mm-demo" class="secondary" type="button">Load demo (80 bars, ~10-pt range)</button>
                <button data-i18n="view.murrey_math.btn.clear" id="mm-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.murrey_math.btn.compute_levels" id="mm-run" class="primary" type="button">Compute levels</button>
            </div>
        </div>

        <div id="mm-errors" class="boot" style="display:none"></div>
        <div id="mm-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.murrey_math.h2.close_murrey_levels">Close + Murrey levels</h2>
            <div id="mm-chart" style="height:340px"></div>
            <p data-i18n="view.murrey_math.hint.cyan_close_yellow_solid_4_8_major_s_r_mid_red_dash" class="muted">Cyan = close. Yellow solid = 4/8 (major S/R mid). Red dashed = 8/8
                ultimate resistance + green dashed = 0/8 ultimate support. Grey dashed = octave
                interior + thin extension levels above/below.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.murrey_math.h2.level_table">Level table</h2>
            <div id="mm-table"></div>
            <p data-i18n="view.murrey_math.hint.all_13_levels_2_8_to_10_8_with_significance_distan" class="muted">All 13 levels (−2/8 to 10/8) with significance + distance from price.
                The two bracketing levels (immediately above and below current price) are highlighted.</p>
        </div>

        <div id="mm-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('mm-demo').addEventListener('click', () => {
        const b = makeDemoBars();
        document.getElementById('mm-bars').value =
            b.map(x => `${x.high} ${x.low} ${x.close}`).join('\n');
    });
    document.getElementById('mm-clear').addEventListener('click', () => {
        document.getElementById('mm-bars').value = '';
    });
    document.getElementById('mm-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.barText = document.getElementById('mm-bars').value;
    state.lookback = parseInt(document.getElementById('mm-lb').value, 10);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('mm-errors');
    errs.style.display = 'none';
    const { bars, errors } = parseBarBlob(state.barText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (bars.length === 0) return;
    }
    const err = validateInputs(bars, state.lookback);
    if (err) { showErr(err); return; }
    let res;
    try {
        res = await api.anlyMurreyMath(buildBody(bars, state.lookback));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!res) { showErr(t('view.murrey_math.err.backend_returned_null_likely_degenerate_price_rang')); return; }
    renderSummary(res, bars);
    renderChart(bars, res);
    renderTable(res);
}

function renderSummary(r, bars) {
    const pos = pricePosition(r.current_price, r.levels || []);
    const { below, above } = bracketingLevels(r.current_price, r.levels || []);
    const nearestLbl = r.nearest_level ? r.nearest_level[0] : '—';
    const nearestVal = r.nearest_level ? r.nearest_level[1] : NaN;
    const nearestSig = significanceOf(nearestLbl);
    document.getElementById('mm-summary').innerHTML = [
        card(t('view.murrey_math.card.bars'),           String(bars.length)),
        card(t('view.murrey_math.card.lookback'),       String(state.lookback)),
        card(t('view.murrey_math.card.current_price'),  fmtN(r.current_price, 2)),
        card(t('view.murrey_math.card.octave_position'), pricePositionLabel(pos), pos === 'lower half' ? 'pos' : pos === 'upper half' ? 'neg' : ''),
        card(t('view.murrey_math.card.bracket_below'),  below ? `${below[0]} @ ${fmtN(below[1], 4)}` : '—', 'pos'),
        card(t('view.murrey_math.card.bracket_above'),  above ? `${above[0]} @ ${fmtN(above[1], 4)}` : '—', 'neg'),
        card(t('view.murrey_math.card.nearest_level'),  `${nearestLbl} @ ${fmtN(nearestVal, 4)}`, nearestSig.cls),
        card(t('view.murrey_math.card.distance_to_nearest'), fmtPct(r.distance_to_nearest_pct)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(bars, r) {
    if (!window.uPlot) return;
    const el = document.getElementById('mm-chart');
    const xs = bars.map((_, i) => i);
    const closes = bars.map(b => b.close);
    // Build a level-line series per Murrey level — each is a constant y.
    const levelSeries = [];
    const levelData = [];
    for (const [lbl, value] of r.levels || []) {
        const sig = significanceOf(lbl);
        const stroke =
            lbl === '0/8' ? '#39ff14' :
            lbl === '8/8' ? '#ff3860' :
            lbl === '4/8' ? '#ffd84a' :
            sig.rank === 'major' ? '#a06bff' :
            sig.rank === 'extended' ? '#444' :
            '#666';
        const dash = lbl === '4/8' ? null : [4, 4];
        const width = sig.rank === 'critical' ? 1.5 : sig.rank === 'major' ? 1.0 : 0.7;
        levelSeries.push({
            label: lbl, stroke, width,
            dash: dash || undefined, points: { show: false },
        });
        levelData.push(xs.map(() => value));
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.bar_num') },
            { label: t('chart.series.close'), stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff14', points: { show: false } },
            ...levelSeries,
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xs, closes, ...levelData], el);
}

function renderTable(r) {
    const wrap = document.getElementById('mm-table');
    const levels = r.levels || [];
    if (!levels.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.murrey_math.empty.levels">No levels.</div>`;
        return;
    }
    const { below, above } = bracketingLevels(r.current_price, levels);
    const belowLbl = below?.[0];
    const aboveLbl = above?.[0];
    // Sort top-down for chart-aligned reading.
    const sorted = [...levels].sort((a, b) => b[1] - a[1]);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.murrey_math.th.level">Level</th><th data-i18n="view.murrey_math.th.price">Price</th><th data-i18n="view.murrey_math.th.significance">Significance</th><th data-i18n="view.murrey_math.th.rank">Rank</th><th data-i18n="view.murrey_math.th.from_current">Δ from current</th><th data-i18n="view.murrey_math.th.bracket">Bracket?</th>
            </tr></thead>
            <tbody>
                ${sorted.map(([lbl, v]) => {
                    const sig = significanceOf(lbl);
                    const delta = v - r.current_price;
                    const bracket = lbl === belowLbl ? '↑ above price' : lbl === aboveLbl ? '↓ below price' : '';
                    return `<tr>
                        <td><strong>${esc(lbl)}</strong></td>
                        <td>${esc(fmtN(v, 4))}</td>
                        <td class="${sig.cls}">${esc(sig.label)}</td>
                        <td>${esc(sig.rank)}</td>
                        <td class="${delta >= 0 ? 'neg' : 'pos'}">${esc(fmtN(delta, 4))}</td>
                        <td>${esc(bracket)}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('mm-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mm-err').style.display = 'none'; }
