// Per-Symbol Slippage view — TCA roll-up across many trades.
//
// Companion to VWAP Slippage: that view drills into one trade vs VWAP;
// this view scans across all symbols to surface the *systematic* problem
// instruments where execution quality is consistently poor.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseRecordBlob, validateInputs, buildBody,
    executionGrade, worstSymbol, bestSymbol,
    makeDemoRecords, fmtBps, fmtPct, fmtN,
} from '../_per_symbol_slippage_inputs.js';

import { t } from '../i18n.js';
let state = { recordsText: '' };

export async function renderPerSymbolSlippage(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.per_symbol_slippage.h1.per_symbol_slippage" class="view-title">// PER-SYMBOL SLIPPAGE</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.per_symbol_slippage.h2.slippage_records">Slippage records</h2>
            <p class="muted">Paste <code>symbol slippage_bps</code> per line. Signed bps —
                positive = trader-favorable (beat benchmark). Demo loads 108 fills
                across 6 symbols spanning the full execution-quality spectrum from
                ETF to micro-cap.</p>
            <textarea id="ps-recs" rows="6" placeholder="AAPL -2.5&#10;SPY 7.0&#10;ILQD -28.0&#10;..."></textarea>
            <div class="inline-form">
                <button data-i18n="view.per_symbol_slippage.btn.load_demo_108_fills_6_symbols" id="ps-demo" class="secondary" type="button">Load demo (108 fills, 6 symbols)</button>
                <button data-i18n="view.per_symbol_slippage.btn.clear" id="ps-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.per_symbol_slippage.btn.aggregate" id="ps-run" class="primary" type="button">Aggregate</button>
            </div>
        </div>

        <div id="ps-errors" class="boot" style="display:none"></div>
        <div id="ps-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.per_symbol_slippage.h2.per_symbol_breakdown">Per-symbol breakdown</h2>
            <div id="ps-table"></div>
            <p data-i18n="view.per_symbol_slippage.hint.sorted_worst_mean_first_poor_terrible_rows_are_whe" class="muted">Sorted worst-mean-first. POOR / TERRIBLE rows are where
                your sizing is bigger than the book can absorb — shrink position there
                or stop trading the name.</p>
        </div>

        <div id="ps-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('ps-demo').addEventListener('click', () => {
        const recs = makeDemoRecords(42);
        document.getElementById('ps-recs').value =
            recs.map(r => `${r.symbol} ${r.slippage_bps}`).join('\n');
    });
    document.getElementById('ps-clear').addEventListener('click', () => {
        document.getElementById('ps-recs').value = '';
    });
    document.getElementById('ps-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.recordsText = document.getElementById('ps-recs').value;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('ps-errors');
    errs.style.display = 'none';
    const { records, errors } = parseRecordBlob(state.recordsText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            `line ${e.line_no}: ${esc(e.message)} — ${esc(e.raw.slice(0, 80))}`).join('<br>');
        const more = errors.length > 8 ? `<br>… and ${errors.length - 8} more.` : '';
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (records.length === 0) return;
    }
    const err = validateInputs(records);
    if (err) { showErr(err); return; }
    let report;
    try {
        report = await api.microPerSymbolSlippage(buildBody(records));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report, records);
    renderTable(report);
}

function renderSummary(report, records) {
    const worst = worstSymbol(report);
    const best  = bestSymbol(report);
    const allBps = records.map(r => r.slippage_bps).filter(Number.isFinite);
    const aggMean = allBps.length ? allBps.reduce((a, b) => a + b, 0) / allBps.length : NaN;
    const beatRate = allBps.length ? allBps.filter(v => v > 0).length / allBps.length : NaN;
    document.getElementById('ps-summary').innerHTML = [
        card(t('view.per_symbol_slippage.card.symbols'),         String(report.length)),
        card(t('view.per_symbol_slippage.card.total_fills'),     String(records.length)),
        card(t('view.per_symbol_slippage.card.mean_slip'),       fmtBps(aggMean), aggMean > 0 ? 'pos' : aggMean < 0 ? 'neg' : ''),
        card(t('view.per_symbol_slippage.card.beat_rate'),       fmtPct(beatRate), beatRate >= 0.5 ? 'pos' : 'neg'),
        card(t('view.per_symbol_slippage.card.worst_symbol'),    worst ? `${worst.symbol} ${fmtBps(worst.mean_bps)}` : '—', 'neg'),
        card(t('view.per_symbol_slippage.card.best_symbol'),     best ? `${best.symbol} ${fmtBps(best.mean_bps)}`   : '—', 'pos'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTable(report) {
    const wrap = document.getElementById('ps-table');
    if (!report.length) {
        wrap.innerHTML = '<div class="muted">No symbols.</div>';
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.per_symbol_slippage.th.symbol">Symbol</th><th data-i18n="view.per_symbol_slippage.th.grade">Grade</th><th data-i18n="view.per_symbol_slippage.th.trades">Trades</th>
                <th data-i18n="view.per_symbol_slippage.th.mean">Mean</th><th data-i18n="view.per_symbol_slippage.th.median">Median</th><th data-i18n="view.per_symbol_slippage.th.stdev">Stdev</th>
                <th data-i18n="view.per_symbol_slippage.th.worst">Worst</th><th data-i18n="view.per_symbol_slippage.th.best">Best</th><th data-i18n="view.per_symbol_slippage.th.beat_rate">Beat rate</th>
            </tr></thead>
            <tbody>
                ${report.map(r => {
                    const grade = executionGrade(r.mean_bps);
                    return `<tr>
                        <td>${esc(r.symbol)}</td>
                        <td class="${grade.cls}">${esc(grade.label)}</td>
                        <td>${esc(fmtN(r.trade_count))}</td>
                        <td class="${r.mean_bps >= 0 ? 'pos' : 'neg'}">${esc(fmtBps(r.mean_bps))}</td>
                        <td class="${r.median_bps >= 0 ? 'pos' : 'neg'}">${esc(fmtBps(r.median_bps))}</td>
                        <td>${esc(fmtBps(r.stdev_bps))}</td>
                        <td class="neg">${esc(fmtBps(r.worst_bps))}</td>
                        <td class="pos">${esc(fmtBps(r.best_bps))}</td>
                        <td>${esc(fmtPct(r.beat_rate))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('ps-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ps-err').style.display = 'none'; }
