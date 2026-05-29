// Momentum Crash Protection view — Daniel & Moskowitz (2016) inverse-vol
// scaling + trailing-cumret crash filter.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_INPUTS,
    parseReturnsBlob, returnsToBlob, validateInputs, buildBody, localManage,
    summarize, cumReturn, maxDrawdown,
    leverageBadge, crashBadge,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtLev, fmtNum, fmtInt,
} from '../_momentum_crash_inputs.js';

let state = { ...makeDemoInput('crash-event') };

export async function renderMomentumCrash(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.mcp.h1.title" class="view-title">// MOMENTUM CRASH PROTECTION</h1>

        <div class="chart-panel" data-context-scope="mcp">
            <h2 data-i18n="view.mcp.h2.returns">Momentum-strategy returns
                <small data-i18n="view.mcp.h2.returns_hint" class="muted">(one per token; raw decimal 0.012 or pct "1.2%"; # comments ignored)</small></h2>
            <textarea id="mcp-blob" rows="6"
                      data-tip="view.mcp.tip.returns"
                      placeholder="0.001 0.002 -0.005 ...">${esc(returnsToBlob(state.momentum_returns))}</textarea>

            <h2 data-i18n="view.mcp.h2.params">Parameters</h2>
            <div class="inline-form">
                <label><span data-i18n="view.mcp.label.vol_lookback">Vol lookback (bars)</span>
                    <input id="mcp-vol-lb" type="number" step="1" min="5" value="${state.vol_lookback}"></label>
                <label><span data-i18n="view.mcp.label.target_vol">Target ann. vol</span>
                    <input id="mcp-target" type="number" step="any" min="0.001" value="${state.target_annualized_vol}"></label>
                <label><span data-i18n="view.mcp.label.periods">Periods / yr</span>
                    <input id="mcp-periods" type="number" step="any" min="1" value="${state.periods_per_year}"></label>
                <label><span data-i18n="view.mcp.label.max_lev">Max leverage</span>
                    <input id="mcp-max-lev" type="number" step="any" min="0.1" value="${state.max_leverage}"></label>
                <label><span data-i18n="view.mcp.label.crash_lb">Crash lookback (bars)</span>
                    <input id="mcp-crash-lb" type="number" step="1" min="1" value="${state.crash_filter_lookback}"></label>
                <label><span data-i18n="view.mcp.label.crash_thr">Crash threshold (decimal)</span>
                    <input id="mcp-crash-thr" type="number" step="any" value="${state.crash_filter_threshold_pct}"></label>
                <button data-i18n="view.mcp.btn.compute" id="mcp-run" class="primary"
                        data-tip="view.mcp.tip.compute" type="button">Manage</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.mcp.btn.demo_normal"     id="mcp-demo-normal"   class="secondary" type="button">Demo: normal regime</button>
                <button data-i18n="view.mcp.btn.demo_low"        id="mcp-demo-low"      class="secondary" type="button">Demo: low vol</button>
                <button data-i18n="view.mcp.btn.demo_high"       id="mcp-demo-high"     class="secondary" type="button">Demo: high vol</button>
                <button data-i18n="view.mcp.btn.demo_crash"      id="mcp-demo-crash"    class="secondary" type="button">Demo: crash event</button>
                <button data-i18n="view.mcp.btn.demo_persistent" id="mcp-demo-pers"     class="secondary" type="button">Demo: persistent crash</button>
                <button data-i18n="view.mcp.btn.demo_mixed"      id="mcp-demo-mixed"    class="secondary" type="button">Demo: mixed regime</button>
                <button data-i18n="view.mcp.btn.demo_short"      id="mcp-demo-short"    class="secondary" type="button">Demo: short lookback</button>
                <button data-i18n="view.mcp.btn.demo_tight"      id="mcp-demo-tight"    class="secondary" type="button">Demo: tight 5% target vol</button>
            </div>
            <p data-i18n="view.mcp.hint.about" class="muted">w_t = min(target_vol / forecast_vol_t, max_leverage). Crash filter zeros leverage when the trailing crash-lookback cumret &lt; threshold. Defaults: 60-day vol, 22-day crash filter, 15% target ann. vol, −20% crash threshold.</p>
        </div>

        <div id="mcp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.mcp.h2.chart">Leverage + cumulative managed vs raw</h2>
            <div id="mcp-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.mcp.h2.table">Per-bar managed series (tail — last 30 bars)</h2>
            <div id="mcp-table"></div>
        </div>

        <div id="mcp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('mcp-blob').value = returnsToBlob(state.momentum_returns);
        document.getElementById('mcp-vol-lb').value = state.vol_lookback;
        document.getElementById('mcp-target').value = state.target_annualized_vol;
        document.getElementById('mcp-periods').value = state.periods_per_year;
        document.getElementById('mcp-max-lev').value = state.max_leverage;
        document.getElementById('mcp-crash-lb').value = state.crash_filter_lookback;
        document.getElementById('mcp-crash-thr').value = state.crash_filter_threshold_pct;
    };
    document.getElementById('mcp-demo-normal').addEventListener('click', () => { loadDemo('normal-regime');   void compute(tok); });
    document.getElementById('mcp-demo-low').addEventListener('click',    () => { loadDemo('low-vol');         void compute(tok); });
    document.getElementById('mcp-demo-high').addEventListener('click',   () => { loadDemo('high-vol');        void compute(tok); });
    document.getElementById('mcp-demo-crash').addEventListener('click',  () => { loadDemo('crash-event');     void compute(tok); });
    document.getElementById('mcp-demo-pers').addEventListener('click',   () => { loadDemo('persistent-crash'); void compute(tok); });
    document.getElementById('mcp-demo-mixed').addEventListener('click',  () => { loadDemo('mixed-regime');    void compute(tok); });
    document.getElementById('mcp-demo-short').addEventListener('click',  () => { loadDemo('short-lookback');  void compute(tok); });
    document.getElementById('mcp-demo-tight').addEventListener('click',  () => { loadDemo('tight-target');    void compute(tok); });
    document.getElementById('mcp-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseReturnsBlob(document.getElementById('mcp-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.mcp.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.momentum_returns           = p.returns;
    state.vol_lookback               = parseInt(document.getElementById('mcp-vol-lb').value, 10) || DEFAULT_INPUTS.vol_lookback;
    state.target_annualized_vol      = Number(document.getElementById('mcp-target').value);
    state.periods_per_year           = Number(document.getElementById('mcp-periods').value);
    state.max_leverage               = Number(document.getElementById('mcp-max-lev').value);
    state.crash_filter_lookback      = parseInt(document.getElementById('mcp-crash-lb').value, 10) || DEFAULT_INPUTS.crash_filter_lookback;
    state.crash_filter_threshold_pct = Number(document.getElementById('mcp-crash-thr').value);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localManage(
        state.momentum_returns, state.vol_lookback, state.target_annualized_vol,
        state.periods_per_year, state.max_leverage,
        state.crash_filter_lookback, state.crash_filter_threshold_pct,
    );
    if (!local) { showErr(t('view.mcp.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.portfolioMomentumCrashProtection(buildBody(state));
    } catch (e) {
        showErr(`${t('view.mcp.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.mcp.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localManage(
        state.momentum_returns, state.vol_lookback, state.target_annualized_vol,
        state.periods_per_year, state.max_leverage,
        state.crash_filter_lookback, state.crash_filter_threshold_pct,
    );
    const parityOk = !!local
        && Math.abs(local.mean_leverage - report.mean_leverage) < 1e-9
        && local.leverages.length === report.leverages.length;
    const s = summarize(report);
    const managed_only = [];
    for (const m of report.managed_returns) if (m != null) managed_only.push(m);
    const managedDD = maxDrawdown(managed_only);
    const rawCum = cumReturn(state.momentum_returns);
    const rawDD = maxDrawdown(state.momentum_returns);
    const lBadge = leverageBadge(report.mean_leverage, state.max_leverage);
    const cBadge = crashBadge(s.crash_frac);
    const localTag = pending ? ` (${t('view.mcp.tag.local')})` : '';
    document.getElementById('mcp-summary').innerHTML = [
        card(t('view.mcp.card.verdict'),     t(lBadge.key) + localTag, lBadge.cls),
        card(t('view.mcp.card.crash_regime'), t(cBadge.key), cBadge.cls),
        card(t('view.mcp.card.bars'),        fmtInt(report.n_observations)),
        card(t('view.mcp.card.populated'),   fmtInt(s.populated)),
        card(t('view.mcp.card.mean_lev'),    fmtLev(report.mean_leverage)),
        card(t('view.mcp.card.max_lev'),     fmtLev(s.max_lev)),
        card(t('view.mcp.card.crash_bars'),  fmtInt(s.crash_bars),
             s.crash_bars > 0 ? 'neg' : ''),
        card(t('view.mcp.card.crash_frac'),  fmtPct(s.crash_frac)),
        card(t('view.mcp.card.managed_ret'), fmtPctSigned(s.total_managed),
             s.total_managed > 0 ? 'pos' : s.total_managed < 0 ? 'neg' : ''),
        card(t('view.mcp.card.raw_ret'),     fmtPctSigned(rawCum),
             rawCum > 0 ? 'pos' : rawCum < 0 ? 'neg' : ''),
        card(t('view.mcp.card.managed_dd'),  fmtPct(managedDD),
             managedDD < -0.10 ? 'neg' : ''),
        card(t('view.mcp.card.raw_dd'),      fmtPct(rawDD),
             rawDD < -0.10 ? 'neg' : ''),
        card(t('view.mcp.card.parity'),
             parityOk ? t('view.mcp.tag.ok') : t('view.mcp.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('mcp-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!report || !report.leverages || report.leverages.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.mcp.empty">${esc(t('view.mcp.empty'))}</div>`;
        return;
    }
    const n = report.leverages.length;
    const xs = [];
    const levs = [];
    const cumManaged = [];
    const cumRaw = [];
    let m = 1, r = 1;
    for (let i = 0; i < n; i++) {
        xs.push(i);
        levs.push(report.leverages[i]);
        const mr = report.managed_returns[i];
        if (mr != null) m *= (1 + mr);
        // Raw cum-return tracks from bar 0; multiplies every bar.
        if (Number.isFinite(state.momentum_returns[i])) r *= (1 + state.momentum_returns[i]);
        cumManaged.push(report.managed_returns[i] != null ? (m - 1) : null);
        cumRaw.push(r - 1);
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: {}, y2: { auto: true } },
        series: [
            { label: t('chart.series.bar') },
            { label: 'leverage',    stroke: '#ffd84a', width: 1.5, points: { show: false } },
            { label: 'cum managed', stroke: '#00e5ff', width: 1.5, points: { show: false }, scale: 'y2' },
            { label: 'cum raw',     stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false }, scale: 'y2' },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
            { side: 1, stroke: '#aab', size: 60, scale: 'y2',
              values: (_u, splits) => splits.map(v => (v * 100).toFixed(0) + '%') },
        ],
        legend: { show: true },
    }, [xs, levs, cumManaged, cumRaw], el);
}

function renderTable(report) {
    const wrap = document.getElementById('mcp-table');
    if (!report || !report.leverages || report.leverages.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.mcp.empty">${esc(t('view.mcp.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, report.leverages.length - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.mcp.col.bar">Bar</th>
                <th data-i18n="view.mcp.col.input">Raw return</th>
                <th data-i18n="view.mcp.col.lev">Leverage</th>
                <th data-i18n="view.mcp.col.crash">Crash filter</th>
                <th data-i18n="view.mcp.col.managed">Managed return</th>
            </tr></thead>
            <tbody>
                ${Array.from({ length: report.leverages.length - start }, (_, k) => {
                    const i = start + k;
                    const rawRet = state.momentum_returns[i];
                    const lev = report.leverages[i];
                    const crashOn = report.crash_filter_active[i];
                    const mgd = report.managed_returns[i];
                    const cls = mgd == null ? '' : mgd > 0 ? 'pos' : mgd < 0 ? 'neg' : '';
                    const crashCls = crashOn === true ? 'neg' : crashOn === false ? 'pos' : '';
                    const crashKey = crashOn === true ? 'view.mcp.crash_cell.on'
                                   : crashOn === false ? 'view.mcp.crash_cell.off'
                                   : 'view.mcp.crash_cell.warmup';
                    return `<tr>
                        <td>${i}</td>
                        <td>${esc(fmtPctSigned(rawRet, 4))}</td>
                        <td>${esc(fmtLev(lev))}</td>
                        <td data-i18n="${esc(crashKey)}" class="${crashCls}">${esc(t(crashKey))}</td>
                        <td class="${cls}">${esc(fmtPctSigned(mgd, 4))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
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
    const el = document.getElementById('mcp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mcp-err').style.display = 'none'; }

// Silence "unused" warnings for re-exported formatters used only via JSX-like esc().
void fmtNum;
