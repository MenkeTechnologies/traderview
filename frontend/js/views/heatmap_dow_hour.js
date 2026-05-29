// Day-of-week × hour-of-day P&L heatmap view. Surfaces the trader's
// best + worst (dow, hour) combinations.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DOW_LABELS,
    parseTradeBlob, validateInputs, buildBody, localBuild, dec,
    maxCellAbs, heatClass, extremeCells, winRate, makeDemoRows,
    fmtUSD, fmtUSDSigned, fmtPct, fmtHour,
} from '../_heatmap_dow_hour_inputs.js';

let state = { rows: makeDemoRows('all-week') };

export async function renderHeatmapDowHour(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.heatmap_dow_hour.h1.title" class="view-title">// DOW × HOUR HEATMAP</h1>

        <div class="chart-panel" data-context-scope="heatmap-dow-hour">
            <h2 data-i18n="view.heatmap_dow_hour.h2.trades">Trade history
                <small data-i18n="view.heatmap_dow_hour.h2.trades_hint" class="muted">(per line: YYYY-MM-DD hour net_pnl  OR  YYYY-MM-DDThh:mm net_pnl)</small></h2>
            <textarea id="hh-rows" rows="8"
                      data-tip="view.heatmap_dow_hour.tip.trades"
                      placeholder="2026-05-25 9 100&#10;2026-05-26 14 -50">${esc(rowsToBlob(state.rows))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.heatmap_dow_hour.btn.build" id="hh-run" class="primary"
                        data-tip="view.heatmap_dow_hour.tip.build" type="button">Build heatmap</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.heatmap_dow_hour.btn.demo_mixed"  id="hh-demo-mixed" class="secondary" type="button">Demo: mixed week</button>
                <button data-i18n="view.heatmap_dow_hour.btn.demo_mon"    id="hh-demo-mon"   class="secondary" type="button">Demo: Monday 9am disaster</button>
                <button data-i18n="view.heatmap_dow_hour.btn.demo_sweet"  id="hh-demo-sweet" class="secondary" type="button">Demo: Tue/Wed 10am sweet spot</button>
                <button data-i18n="view.heatmap_dow_hour.btn.demo_weekend" id="hh-demo-wknd" class="secondary" type="button">Demo: weekend crypto</button>
                <button data-i18n="view.heatmap_dow_hour.btn.demo_full"   id="hh-demo-full"  class="secondary" type="button">Demo: full Mon-Fri 9-16</button>
            </div>
            <p data-i18n="view.heatmap_dow_hour.hint.about" class="muted">7×24 grid. Rows = day of week (Sun → Sat). Cols = hour of day (0 → 23 UTC). Cell color = net P&L (green = winning, red = losing, blank = no trades).</p>
        </div>

        <div id="hh-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.heatmap_dow_hour.h2.grid">P&L heatmap</h2>
            <div id="hh-grid"></div>
        </div>

        <div id="hh-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state.rows = makeDemoRows(k);
        document.getElementById('hh-rows').value = rowsToBlob(state.rows);
    };
    document.getElementById('hh-demo-mixed').addEventListener('click',  () => loadDemo('mixed'));
    document.getElementById('hh-demo-mon').addEventListener('click',    () => loadDemo('monday-disaster'));
    document.getElementById('hh-demo-sweet').addEventListener('click',  () => loadDemo('sweet-spot'));
    document.getElementById('hh-demo-wknd').addEventListener('click',   () => loadDemo('weekend-crypto'));
    document.getElementById('hh-demo-full').addEventListener('click',   () => loadDemo('all-week'));
    document.getElementById('hh-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function rowsToBlob(rows) {
    return rows.map(r => `${r.date} ${r.hour} ${r.net_pnl}`).join('\n');
}

function readInputs() {
    const p = parseTradeBlob(document.getElementById('hh-rows').value);
    if (p.errors.length) {
        showErr(`${t('view.heatmap_dow_hour.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.rows = p.rows;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.rows);
    if (err) { showErr(err); return; }
    const local = localBuild(state.rows);
    renderSummary(local, true);
    renderGrid(local);
    let resp;
    try {
        resp = await api.heatmapDowHour(buildBody(state.rows));
    } catch (e) {
        showErr(`${t('view.heatmap_dow_hour.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    const normalized = {
        ...resp,
        total_pnl: dec(resp.total_pnl),
        cells: (resp.cells || local.cells).map(row =>
            row.map(c => ({ ...c, net_pnl: dec(c.net_pnl) }))),
    };
    renderSummary(normalized, false);
    renderGrid(normalized);
}

function renderSummary(report, pending) {
    const local = localBuild(state.rows);
    const parityOk = report.total_trades === local.total_trades
                  && Math.abs(report.total_pnl - local.total_pnl) < 1e-6;
    const { best, worst } = extremeCells(report);
    const localTag = pending ? ` (${t('view.heatmap_dow_hour.tag.local')})` : '';
    document.getElementById('hh-summary').innerHTML = [
        card(t('view.heatmap_dow_hour.card.total_trades'),
             String(report.total_trades) + localTag),
        card(t('view.heatmap_dow_hour.card.total_pnl'),
             fmtUSDSigned(report.total_pnl),
             report.total_pnl >= 0 ? 'pos' : 'neg'),
        card(t('view.heatmap_dow_hour.card.best_cell'),
             best ? `${t('common.dow.' + DOW_LABELS[best.dow].toLowerCase())} ${fmtHour(best.hour)} ${fmtUSDSigned(best.net_pnl)}` : '—',
             best && best.net_pnl > 0 ? 'pos' : ''),
        card(t('view.heatmap_dow_hour.card.worst_cell'),
             worst ? `${t('common.dow.' + DOW_LABELS[worst.dow].toLowerCase())} ${fmtHour(worst.hour)} ${fmtUSDSigned(worst.net_pnl)}` : '—',
             worst && worst.net_pnl < 0 ? 'neg' : ''),
        card(t('view.heatmap_dow_hour.card.dow_count'), '7'),
        card(t('view.heatmap_dow_hour.card.hour_count'), '24'),
        card(t('view.heatmap_dow_hour.card.parity'),
             parityOk ? t('view.heatmap_dow_hour.tag.ok') : t('view.heatmap_dow_hour.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderGrid(report) {
    const wrap = document.getElementById('hh-grid');
    const maxAbs = maxCellAbs(report);
    if (report.total_trades === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.heatmap_dow_hour.empty">${esc(t('view.heatmap_dow_hour.empty'))}</div>`;
        return;
    }
    const headerRow = `
        <tr>
            <th></th>
            ${Array.from({ length: 24 }, (_, h) =>
                `<th class="muted" style="font-size:10px">${esc(fmtHour(h))}</th>`).join('')}
        </tr>`;
    const bodyRows = report.cells.map((row, d) => `
        <tr>
            <td data-i18n="common.dow.${esc(DOW_LABELS[d].toLowerCase())}"
                style="font-weight:bold">${esc(DOW_LABELS[d])}</td>
            ${row.map(cell => {
                const pnl = dec(cell.net_pnl);
                const klass = heatClass(pnl, maxAbs);
                const title = cell.trades > 0
                    ? `${cell.trades} trades · ${fmtUSDSigned(pnl)} · ${fmtPct(winRate(cell))} win`
                    : '';
                return `<td class="${klass}"
                    title="${esc(title)}"
                    style="font-size:10px;padding:4px;text-align:center;min-width:36px">${
                    cell.trades > 0 ? esc(fmtUSDSigned(pnl)) : ''
                }</td>`;
            }).join('')}
        </tr>`).join('');
    wrap.innerHTML = `
        <table class="lq-table" style="font-size:10px">
            <thead>${headerRow}</thead>
            <tbody>${bodyRows}</tbody>
        </table>
    `;
    void fmtUSD;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('hh-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('hh-err').style.display = 'none'; }
