// Open Type view — Dalton's Auction Market Theory opening classifier.
//
// First-hour structural read: did the market open driving, testing,
// rejecting, or auctioning vs prior-day extremes / value area?
// Each verdict carries different intraday trading guidance.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    validateInputs, buildBody, typeBadge,
    makeDemoData, chartSpan, fmtN, yesNo,
} from '../_open_type_inputs.js';

import { t } from '../i18n.js';
let state = { params: makeDemoData('auction') };

export async function renderOpenType(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.open_type.h1.open_type_dalton_amt" class="view-title">// OPEN TYPE · DALTON AMT</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.open_type.h2.today_s_opening_range_first_30_60_min">Today's opening range (first 30-60 min)</h2>
            <div class="inline-form">
                <label><span data-i18n="view.open_type.label.open_price">Open price</span> <input id="ot-open" type="number" step="any" min="0" value="${state.params.open_price}"></label>
                <label><span data-i18n="view.open_type.label.or_high">OR high</span>   <input id="ot-orh"  type="number" step="any" min="0" value="${state.params.opening_range_high}"></label>
                <label><span data-i18n="view.open_type.label.or_low">OR low</span>    <input id="ot-orl"  type="number" step="any" min="0" value="${state.params.opening_range_low}"></label>
                <label><span data-i18n="view.open_type.label.or_close">OR close</span>  <input id="ot-orc"  type="number" step="any" min="0" value="${state.params.opening_range_close}"></label>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.open_type.h2.prior_day_reference">Prior-day reference</h2>
            <div class="inline-form">
                <label><span data-i18n="view.open_type.label.prior_high">Prior high</span>   <input id="ot-ph"  type="number" step="any" min="0" value="${state.params.prior_day_high}"></label>
                <label><span data-i18n="view.open_type.label.prior_low">Prior low</span>    <input id="ot-pl"  type="number" step="any" min="0" value="${state.params.prior_day_low}"></label>
                <label><span data-i18n="view.open_type.label.vah">VAH (Value Area High)</span> <input id="ot-vah" type="number" step="any" min="0" value="${state.params.prior_day_vah}"></label>
                <label><span data-i18n="view.open_type.label.val">VAL (Value Area Low)</span>  <input id="ot-val" type="number" step="any" min="0" value="${state.params.prior_day_val}"></label>
            </div>
            <div class="inline-form">
                <button data-i18n="view.open_type.btn.demo_drive_up" id="ot-demo-drive"   class="secondary" type="button">Demo: Drive Up</button>
                <button data-i18n="view.open_type.btn.demo_test_drive_up" id="ot-demo-test"    class="secondary" type="button">Demo: Test Drive Up</button>
                <button data-i18n="view.open_type.btn.demo_rejection_reverse" id="ot-demo-reject"  class="secondary" type="button">Demo: Rejection Reverse</button>
                <button data-i18n="view.open_type.btn.demo_auction" id="ot-demo-auction" class="secondary" type="button">Demo: Auction</button>
                <button data-i18n="view.open_type.btn.classify" id="ot-run" class="primary" type="button">Classify</button>
            </div>
            <p data-i18n="view.open_type.hint.vah_val_the_70_volume_weighted_value_area_from_the" class="muted">VAH/VAL = the 70% volume-weighted value area from the prior session.
                Most market-profile platforms (Sierra, Bookmap) print these as horizontal lines
                on yesterday's profile.</p>
        </div>

        <div id="ot-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.open_type.h2.reference_levels_schematic">Reference levels schematic</h2>
            <div id="ot-chart" style="height:280px"></div>
            <p data-i18n="view.open_type.hint.yellow_prior_h_l_extreme_range_cyan_vah_val_value_" class="muted">Yellow = prior H/L (extreme range). Cyan = VAH/VAL (value area).
                Magenta = opening range H/L. Red dot = open price. Green dot = OR close.
                Distance + direction visually reveals the verdict.</p>
        </div>

        <div id="ot-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.params = makeDemoData(kind);
        for (const [id, k] of [
            ['ot-open',  'open_price'],
            ['ot-orh',   'opening_range_high'],
            ['ot-orl',   'opening_range_low'],
            ['ot-orc',   'opening_range_close'],
            ['ot-ph',    'prior_day_high'],
            ['ot-pl',    'prior_day_low'],
            ['ot-vah',   'prior_day_vah'],
            ['ot-val',   'prior_day_val'],
        ]) {
            document.getElementById(id).value = state.params[k];
        }
    };
    document.getElementById('ot-demo-drive').addEventListener('click', () => loadDemo('drive-up'));
    document.getElementById('ot-demo-test').addEventListener('click', () => loadDemo('test-drive-up'));
    document.getElementById('ot-demo-reject').addEventListener('click', () => loadDemo('rejection-up'));
    document.getElementById('ot-demo-auction').addEventListener('click', () => loadDemo('auction'));
    document.getElementById('ot-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
    readInputs(); void compute(tok);
}

function readInputs() {
    const get = id => Number(document.getElementById(id).value);
    state.params = {
        open_price:          get('ot-open'),
        opening_range_high:  get('ot-orh'),
        opening_range_low:   get('ot-orl'),
        opening_range_close: get('ot-orc'),
        prior_day_high:      get('ot-ph'),
        prior_day_low:       get('ot-pl'),
        prior_day_vah:       get('ot-vah'),
        prior_day_val:       get('ot-val'),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.params);
    if (err) { showErr(err); return; }
    let res;
    try {
        res = await api.discOpenType(buildBody(state.params));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e })); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res);
    renderChart(state.params);
}

function renderSummary(r) {
    const badge = typeBadge(r.open_type);
    document.getElementById('ot-summary').innerHTML = [
        card(t('view.open_type.card.open_type'),         badge.label, badge.cls),
        card(t('view.open_type.card.above_prior_high'), yesNo(r.above_prior_high),    r.above_prior_high ? 'pos' : ''),
        card(t('view.open_type.card.below_prior_low'),  yesNo(r.below_prior_low),     r.below_prior_low ? 'neg' : ''),
        card(t('view.open_type.card.inside_prior_value'), yesNo(r.inside_prior_value), r.inside_prior_value ? '' : ''),
        card(t('view.open_type.card.action'),            badge.hint),
        card(t('view.open_type.card.backend_note'),      r.note || '—'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(p) {
    if (!window.uPlot) return;
    const el = document.getElementById('ot-chart');
    const { min, max } = chartSpan(p);
    const xs = [0, 1, 2, 3, 4, 5];   // time axis just for visual spread
    // Horizontal reference lines.
    const ph   = xs.map(() => p.prior_day_high);
    const pl   = xs.map(() => p.prior_day_low);
    const vah  = xs.map(() => p.prior_day_vah);
    const val  = xs.map(() => p.prior_day_val);
    const orh  = xs.map(() => p.opening_range_high);
    const orl  = xs.map(() => p.opening_range_low);
    // Marker points: open price at x=1, OR close at x=4.
    const openMark = xs.map((_, i) => i === 1 ? p.open_price : null);
    const closMark = xs.map((_, i) => i === 4 ? p.opening_range_close : null);

    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: {}, y: { range: [min, max] } },
        series: [
            { label: 't' },
            { label: t('view.open_type.series.prior_high'), stroke: '#ffd84a', width: 1.5, points: { show: false } },
            { label: t('view.open_type.series.prior_low'),  stroke: '#ffd84a', width: 1.5, dash: [4, 4], points: { show: false } },
            { label: 'VAH',        stroke: '#00e5ff', width: 1.0, points: { show: false } },
            { label: 'VAL',        stroke: '#00e5ff', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('view.open_type.series.or_high'),    stroke: '#ff3860', width: 1.0, points: { show: false } },
            { label: t('view.open_type.series.or_low'),     stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('view.open_type.series.open'),       stroke: '#fff',    width: 0, points: { show: true, size: 12, stroke: '#fff',    fill: '#ff3860' } },
            { label: t('view.open_type.series.or_close'),   stroke: '#fff',    width: 0, points: { show: true, size: 12, stroke: '#fff',    fill: '#39ff14' } },
        ],
        axes: [{ stroke: '#aab', size: 24 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xs, ph, pl, vah, val, orh, orl, openMark, closMark], el);
    void fmtN;
}

function showErr(msg) {
    const el = document.getElementById('ot-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ot-err').style.display = 'none'; }
