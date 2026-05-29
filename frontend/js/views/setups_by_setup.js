// Setups-by-setup view — rolls trades up by setup tag and ranks them.
// Per-setup: trades count, wins/losses/scratches, win-rate, profit
// factor, avg R, expectancy. Sorted by net_pnl DESC so winning setups
// float to the top.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSetupTradeBlob, validateInputs, buildBody, localAnalyze,
    dec, setupBadge, makeDemoRows,
    fmtUSD, fmtUSDSigned, fmtPct, fmtPF, fmtR,
} from '../_setups_by_setup_inputs.js';

let state = { rows: makeDemoRows('mixed') };

export async function renderSetupsBySetup(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// SETUPS — RANKED BY P&amp;L</h1>

        <div class="chart-panel">
            <h2>Trades (per line: <code>setup net_pnl [risk_amount]</code>; "-" = untagged)</h2>
            <textarea id="sbs-blob" rows="10" placeholder="orb 500 100&#10;abcd -150 100&#10;- 999 100   # untagged, excluded">${esc(rowsToBlob(state.rows))}</textarea>
            <div class="inline-form">
                <button id="sbs-run" class="primary" type="button">Analyze</button>
                <button id="sbs-demo-mixed"    class="secondary" type="button">Demo: 3 setups mixed</button>
                <button id="sbs-demo-winner"   class="secondary" type="button">Demo: single winner</button>
                <button id="sbs-demo-loser"    class="secondary" type="button">Demo: single loser</button>
                <button id="sbs-demo-untag"    class="secondary" type="button">Demo: with untagged</button>
                <button id="sbs-demo-scratch"  class="secondary" type="button">Demo: all scratches</button>
            </div>
            <p class="muted">Risk amount enables R-multiple. Net P&L &gt; 0 = win, &lt; 0 = loss, == 0 = scratch. Untagged trades ("-") are sent to the backend but excluded from any setup bucket — matches backend "no setup tag → not in the catalog".</p>
        </div>

        <div id="sbs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Per-setup leaderboard</h2>
            <div id="sbs-stats"></div>
        </div>

        <div id="sbs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.rows = makeDemoRows(kind);
        document.getElementById('sbs-blob').value = rowsToBlob(state.rows);
    };
    document.getElementById('sbs-demo-mixed').addEventListener('click',   () => loadDemo('mixed'));
    document.getElementById('sbs-demo-winner').addEventListener('click',  () => loadDemo('single-winner'));
    document.getElementById('sbs-demo-loser').addEventListener('click',   () => loadDemo('single-loser'));
    document.getElementById('sbs-demo-untag').addEventListener('click',   () => loadDemo('with-untagged'));
    document.getElementById('sbs-demo-scratch').addEventListener('click', () => loadDemo('all-scratches'));
    document.getElementById('sbs-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function rowsToBlob(rows) {
    return rows.map(r => {
        const risk = r.risk_amount != null ? ` ${r.risk_amount}` : '';
        return `${r.setup} ${r.net_pnl}${risk}`;
    }).join('\n');
}

function readInputs() {
    const parsed = parseSetupTradeBlob(document.getElementById('sbs-blob').value);
    if (parsed.errors.length) {
        showErr(`Parse errors: ${parsed.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; ')}`);
        return;
    }
    hideErr();
    state.rows = parsed.rows;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.rows);
    if (err) { showErr(err); return; }
    const local = localAnalyze(state.rows);
    renderSummary(local, true);
    renderStats(local, true);
    let resp;
    try {
        resp = await api.setupsBySetup(buildBody(state.rows));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    // Backend returns Decimals as strings — coerce to numbers.
    const normalized = resp.map(s => ({
        ...s,
        net_pnl: dec(s.net_pnl), gross_pnl: dec(s.gross_pnl), fees: dec(s.fees),
        avg_pnl: dec(s.avg_pnl), avg_win: dec(s.avg_win), avg_loss: dec(s.avg_loss),
        expectancy: dec(s.expectancy),
        largest_win: dec(s.largest_win), largest_loss: dec(s.largest_loss),
    }));
    renderSummary(normalized, false);
    renderStats(normalized, false);
}

function renderSummary(stats, pending) {
    const local = localAnalyze(state.rows);
    const parity = statsArrEq(stats, local);
    const totalTrades = stats.reduce((a, s) => a + s.trades, 0);
    const totalNet    = stats.reduce((a, s) => a + s.net_pnl, 0);
    const totalWins   = stats.reduce((a, s) => a + s.wins, 0);
    const totalLosses = stats.reduce((a, s) => a + s.losses, 0);
    const untagged    = state.rows.filter(r => r.setup === '-' || r.setup === '').length;
    const winRate     = totalTrades > 0 ? totalWins / totalTrades : 0;
    const best  = stats.length > 0 ? stats[0] : null;
    const worst = stats.length > 0 ? stats[stats.length - 1] : null;
    document.getElementById('sbs-summary').innerHTML = [
        card('Setups',         String(stats.length) + (pending ? ' (local)' : '')),
        card('Tagged trades',  String(totalTrades)),
        card('Untagged (skipped)', String(untagged),
            untagged > 0 ? 'neg' : ''),
        card('Total net P&L',  fmtUSDSigned(totalNet),
            totalNet >= 0 ? 'pos' : 'neg'),
        card('Total win rate', fmtPct(winRate),
            winRate >= 0.5 ? 'pos' : 'neg'),
        card('Wins / Losses',  `${totalWins} / ${totalLosses}`,
            totalWins > totalLosses ? 'pos' : 'neg'),
        card('Best setup',     best ? `${best.setup}: ${fmtUSDSigned(best.net_pnl)}` : '—',
            best && best.net_pnl >= 0 ? 'pos' : 'neg'),
        card('Worst setup',    worst ? `${worst.setup}: ${fmtUSDSigned(worst.net_pnl)}` : '—',
            worst && worst.net_pnl >= 0 ? 'pos' : 'neg'),
        card('Local parity',   parity ? 'OK' : 'DIVERGED', parity ? 'pos' : 'neg'),
    ].join('');
}

function statsArrEq(a, b) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        if (a[i].setup !== b[i].setup) return false;
        if (Math.abs(a[i].net_pnl - b[i].net_pnl) > 1e-6) return false;
        if (a[i].trades !== b[i].trades) return false;
        if (a[i].wins !== b[i].wins) return false;
        if (a[i].losses !== b[i].losses) return false;
    }
    return true;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderStats(stats) {
    const wrap = document.getElementById('sbs-stats');
    if (!stats.length) { wrap.innerHTML = '<div class="muted">No tagged trades.</div>'; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th>Setup</th><th>Verdict</th>
                <th>Trades</th><th>W/L/S</th><th>Win%</th>
                <th>Net P&L</th><th>Expectancy</th><th>Avg win</th><th>Avg loss</th>
                <th>Profit factor</th><th>Avg R</th>
                <th>Largest win</th><th>Largest loss</th>
            </tr></thead>
            <tbody>
                ${stats.map((s, i) => {
                    const badge = setupBadge(s);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td><strong>${esc(s.setup)}</strong></td>
                        <td class="${badge.cls}">${esc(badge.label)}</td>
                        <td>${s.trades}</td>
                        <td>${s.wins}/${s.losses}/${s.scratches}</td>
                        <td class="${s.win_rate >= 0.5 ? 'pos' : 'neg'}">${esc(fmtPct(s.win_rate))}</td>
                        <td class="${s.net_pnl >= 0 ? 'pos' : 'neg'}">${esc(fmtUSDSigned(s.net_pnl))}</td>
                        <td class="${s.expectancy >= 0 ? 'pos' : 'neg'}">${esc(fmtUSDSigned(s.expectancy))}</td>
                        <td class="pos">${esc(fmtUSDSigned(s.avg_win))}</td>
                        <td class="neg">${esc(fmtUSDSigned(s.avg_loss))}</td>
                        <td class="${s.profit_factor >= 1 ? 'pos' : 'neg'}">${esc(fmtPF(s.profit_factor))}</td>
                        <td class="${s.avg_r >= 0 ? 'pos' : 'neg'}">${esc(fmtR(s.avg_r))}</td>
                        <td class="pos">${esc(fmtUSD(s.largest_win))}</td>
                        <td class="neg">${esc(fmtUSDSigned(s.largest_loss))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('sbs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('sbs-err').style.display = 'none'; }
