// Tax-loss-harvest suggester view. Finds open losers, ranks by loss
// size, flags wash-sale risk (±30 days) and $3k capital-loss cap
// (unless MTM-elected). Year-end tax planning tool.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseLoserBlob, parseRecentBuyBlob, validateInputs, buildBody,
    localSuggest, dec, harvestBadge, todayIso,
    makeDemoLosers, makeDemoRecentBuys,
    fmtUSD, fmtUSDSigned, fmtNum, fmtBool,
} from '../_tax_loss_harvest_inputs.js';

let state = {
    losers: makeDemoLosers('mixed'),
    recent_buys: makeDemoRecentBuys('mixed'),
    today: todayIso(),
    realized_loss_ytd: 0,
    mtm_elected: false,
};

export async function renderTaxLossHarvest(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.tax_loss_harvest.h1.title" class="view-title">// TAX-LOSS HARVEST</h1>

        <div class="chart-panel" data-context-scope="tax-loss-harvest">
            <h2 data-i18n="view.tax_loss_harvest.h2.losers">Open losers
                <small data-i18n="view.tax_loss_harvest.h2.losers_hint" class="muted">(per line: SYMBOL qty avg_cost current_price)</small></h2>
            <textarea id="tlh-losers" rows="5"
                      data-tip="view.tax_loss_harvest.tip.losers"
                      placeholder="AAPL 100 150 140&#10;TSLA 10 300 250">${esc(losersToBlob(state.losers))}</textarea>

            <h2 data-i18n="view.tax_loss_harvest.h2.recent">Recent buys
                <small data-i18n="view.tax_loss_harvest.h2.recent_hint" class="muted">(per line: SYMBOL YYYY-MM-DD)</small></h2>
            <textarea id="tlh-buys" rows="3"
                      data-tip="view.tax_loss_harvest.tip.recent"
                      placeholder="AAPL 2026-05-15">${esc(buysToBlob(state.recent_buys))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.tax_loss_harvest.label.today">Today</span>
                    <input id="tlh-today" type="date" value="${esc(state.today)}"></label>
                <label><span data-i18n="view.tax_loss_harvest.label.ytd">YTD realized loss ($)</span>
                    <input id="tlh-ytd" type="number" step="any" value="${state.realized_loss_ytd}"></label>
                <label><span data-i18n="view.tax_loss_harvest.label.mtm">MTM §475(f) elected?</span>
                    <input id="tlh-mtm" type="checkbox" ${state.mtm_elected ? 'checked' : ''}></label>
                <button data-i18n="view.tax_loss_harvest.btn.suggest" id="tlh-run" class="primary"
                        data-tip="view.tax_loss_harvest.tip.suggest" type="button">Suggest</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.tax_loss_harvest.btn.demo_mixed"    id="tlh-demo-mix"   class="secondary" type="button">Demo: mixed (3 losers)</button>
                <button data-i18n="view.tax_loss_harvest.btn.demo_wash"     id="tlh-demo-wash"  class="secondary" type="button">Demo: wash-sale risk</button>
                <button data-i18n="view.tax_loss_harvest.btn.demo_3k"       id="tlh-demo-3k"    class="secondary" type="button">Demo: exceeds $3k cap</button>
                <button data-i18n="view.tax_loss_harvest.btn.demo_winners"  id="tlh-demo-win"   class="secondary" type="button">Demo: winners only (nothing to harvest)</button>
                <button data-i18n="view.tax_loss_harvest.btn.demo_three"    id="tlh-demo-three" class="secondary" type="button">Demo: BIG / MID / TINY sort</button>
            </div>
            <p data-i18n="view.tax_loss_harvest.hint.about" class="muted">Sorts genuine losers by loss size. Flags ±30-day wash-sale risk (§1091) and $3k capital-loss cap (unless MTM elected). YTD context lets the cap detector see your running tally.</p>
        </div>

        <div id="tlh-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.tax_loss_harvest.h2.candidates">Harvest candidates (largest loss first)</h2>
            <div id="tlh-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.tax_loss_harvest.h2.losses_chart">Loss per candidate (red = wash risk)</h2>
            <div id="tlh-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.tax_loss_harvest.h2.qty_chart">Lot size per candidate (shares)</h2>
            <div id="tlh-qty-chart" style="width:100%;height:200px"></div>
        </div>

        <div id="tlh-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state.losers = makeDemoLosers(k);
        state.recent_buys = makeDemoRecentBuys(k, state.today);
        state.realized_loss_ytd = k === 'exceeds-3k' ? 2500 : 0;
        document.getElementById('tlh-losers').value = losersToBlob(state.losers);
        document.getElementById('tlh-buys').value   = buysToBlob(state.recent_buys);
        document.getElementById('tlh-ytd').value    = state.realized_loss_ytd;
    };
    document.getElementById('tlh-demo-mix').addEventListener('click',   () => loadDemo('mixed'));
    document.getElementById('tlh-demo-wash').addEventListener('click',  () => loadDemo('wash-sale'));
    document.getElementById('tlh-demo-3k').addEventListener('click',    () => loadDemo('exceeds-3k'));
    document.getElementById('tlh-demo-win').addEventListener('click',   () => loadDemo('winners-only'));
    document.getElementById('tlh-demo-three').addEventListener('click', () => loadDemo('big-three'));
    document.getElementById('tlh-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function losersToBlob(losers) {
    return losers.map(l => `${l.symbol} ${l.qty} ${l.avg_cost} ${l.current_price}`).join('\n');
}

function buysToBlob(buys) {
    return buys.map(b => `${b.symbol} ${b.executed_at}`).join('\n');
}

function readInputs() {
    const pl = parseLoserBlob(document.getElementById('tlh-losers').value);
    const pb = parseRecentBuyBlob(document.getElementById('tlh-buys').value);
    const errs = [
        ...pl.errors.map(e => `loser[${e.line_no}] ${e.message}`),
        ...pb.errors.map(e => `buy[${e.line_no}] ${e.message}`),
    ];
    if (errs.length) { showErr(errs.slice(0, 4).join('; ')); return; }
    hideErr();
    state.losers = pl.losers;
    state.recent_buys = pb.buys;
    state.today = document.getElementById('tlh-today').value;
    state.realized_loss_ytd = Number(document.getElementById('tlh-ytd').value);
    state.mtm_elected = document.getElementById('tlh-mtm').checked;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.losers, state.recent_buys, state.today,
                               state.realized_loss_ytd, state.mtm_elected);
    if (err) { showErr(err); return; }
    const local = localSuggest(state.losers, state.recent_buys, state.today,
                                state.realized_loss_ytd, state.mtm_elected);
    renderSummary(local, true);
    renderTable(local);
    renderLossesChart(local);
    renderQtyChart(local);
    let resp;
    try {
        resp = await api.calcTaxLossHarvest(buildBody(
            state.losers, state.recent_buys, state.today,
            state.realized_loss_ytd, state.mtm_elected));
    } catch (e) {
        showErr(`${t('view.tax_loss_harvest.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    const normalized = {
        ...resp,
        total_available_loss: dec(resp.total_available_loss),
        safe_harvest_loss:    dec(resp.safe_harvest_loss),
        candidates: (resp.candidates || []).map(c => ({
            ...c,
            qty:             dec(c.qty),
            unrealized_loss: dec(c.unrealized_loss),
        })),
    };
    renderSummary(normalized, false);
    renderTable(normalized);
    renderLossesChart(normalized);
    renderQtyChart(normalized);
}

function renderQtyChart(report) {
    const el = document.getElementById('tlh-qty-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (report?.candidates || []).filter(c => Number.isFinite(Number(c.qty)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.tax_loss_harvest.empty_qty_chart">${esc(t('view.tax_loss_harvest.empty_qty_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.qty) - Number(a.qty));
    const labels = rows.map(c => c.symbol);
    const xs = labels.map((_, i) => i + 1);
    const ys = rows.map(c => Number(c.qty));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.tax_loss_harvest.chart.symbol') },
            { label: t('view.tax_loss_harvest.chart.qty'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderLossesChart(report) {
    const el = document.getElementById('tlh-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (report?.candidates || []).filter(c => Number.isFinite(Number(c.unrealized_loss)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.tax_loss_harvest.empty_chart">${esc(t('view.tax_loss_harvest.empty_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.unrealized_loss) - Number(a.unrealized_loss));
    const labels = rows.map(c => c.symbol);
    const xs = labels.map((_, i) => i + 1);
    const wash = rows.map(c => c.wash_sale_risk  ? Number(c.unrealized_loss) : null);
    const safe = rows.map(c => !c.wash_sale_risk ? Number(c.unrealized_loss) : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.tax_loss_harvest.chart.symbol') },
            { label: t('view.tax_loss_harvest.chart.wash'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 14, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.tax_loss_harvest.chart.safe'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, wash, safe], el);
}

function renderSummary(report, pending) {
    const badge = harvestBadge(report.safe_harvest_loss);
    const local = localSuggest(state.losers, state.recent_buys, state.today,
                                state.realized_loss_ytd, state.mtm_elected);
    const parityOk = Math.abs(report.total_available_loss - local.total_available_loss) < 1e-6
                  && Math.abs(report.safe_harvest_loss - local.safe_harvest_loss) < 1e-6;
    const localTag = pending ? ` (${t('view.tax_loss_harvest.tag.local')})` : '';
    const washCount = report.candidates.filter(c => c.wash_sale_risk).length;
    const capCount  = report.candidates.filter(c => c.exceeds_3k_cap).length;
    document.getElementById('tlh-summary').innerHTML = [
        card(t('view.tax_loss_harvest.card.verdict'),
             t(badge.key) + localTag, badge.cls),
        card(t('view.tax_loss_harvest.card.candidates'),
             String(report.candidates.length)),
        card(t('view.tax_loss_harvest.card.total_loss'),
             fmtUSD(report.total_available_loss), 'neg'),
        card(t('view.tax_loss_harvest.card.safe_loss'),
             fmtUSD(report.safe_harvest_loss),
             report.safe_harvest_loss > 0 ? 'pos' : ''),
        card(t('view.tax_loss_harvest.card.wash_count'), String(washCount),
             washCount > 0 ? 'neg' : 'pos'),
        card(t('view.tax_loss_harvest.card.cap_count'), String(capCount),
             capCount > 0 ? 'neg' : ''),
        card(t('view.tax_loss_harvest.card.mtm'),
             state.mtm_elected ? t('view.tax_loss_harvest.tag.yes') : t('view.tax_loss_harvest.tag.no')),
        card(t('view.tax_loss_harvest.card.parity'),
             parityOk ? t('view.tax_loss_harvest.tag.ok') : t('view.tax_loss_harvest.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(report) {
    const wrap = document.getElementById('tlh-table');
    if (!report.candidates || report.candidates.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.tax_loss_harvest.empty">${esc(t('view.tax_loss_harvest.empty'))}</div>`;
        return;
    }
    // Build per-candidate note. Backend's note is a raw string; we either
    // show it OR translate via the local report's note_key (only available
    // on local results). Backend wins when present.
    const local = localSuggest(state.losers, state.recent_buys, state.today,
                                state.realized_loss_ytd, state.mtm_elected);
    const localByVid = new Map(local.candidates.map(c => [c.symbol, c]));
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.tax_loss_harvest.col.symbol">Symbol</th>
                <th data-i18n="view.tax_loss_harvest.col.qty">Qty</th>
                <th data-i18n="view.tax_loss_harvest.col.loss">Unrealized loss</th>
                <th data-i18n="view.tax_loss_harvest.col.wash">Wash risk</th>
                <th data-i18n="view.tax_loss_harvest.col.cap">Exceeds $3k</th>
                <th data-i18n="view.tax_loss_harvest.col.note">Note</th>
            </tr></thead>
            <tbody>
                ${report.candidates.map(c => {
                    const localC = localByVid.get(c.symbol);
                    const noteText = c.note || (localC ? t(localC.note_key) : '');
                    return `<tr>
                        <td><strong>${esc(c.symbol)}</strong></td>
                        <td>${esc(fmtNum(c.qty, 4))}</td>
                        <td class="neg">${esc(fmtUSD(c.unrealized_loss))}</td>
                        <td class="${c.wash_sale_risk ? 'neg' : ''}">${esc(fmtBool(c.wash_sale_risk))}</td>
                        <td class="${c.exceeds_3k_cap ? 'neg' : ''}">${esc(fmtBool(c.exceeds_3k_cap))}</td>
                        <td class="muted">${esc(noteText)}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    void fmtUSDSigned;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('tlh-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('tlh-err').style.display = 'none'; }
