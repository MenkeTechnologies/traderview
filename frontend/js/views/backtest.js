// Strategy backtester — pick a preset + symbol, run, see equity curve + trades.
import { api } from '../api.js';
import { equityChart } from '../charts.js';
import { esc, fmt, fmtDateTime } from '../util.js';

const PRESETS = [
    { id: 'sma_cross', label: 'SMA crossover',
      defaults: { fast: 20, slow: 50 } },
    { id: 'rsi_reversion', label: 'RSI mean reversion',
      defaults: { period: 14, oversold: 30, overbought: 70 } },
    { id: 'bollinger_breakout', label: 'Bollinger breakout',
      defaults: { period: 20, k: 2 } },
    { id: 'macd_cross', label: 'MACD crossover', defaults: {} },
];

export async function renderBacktest(mount) {
    mount.innerHTML = `
        <h1 class="view-title">// STRATEGY BACKTEST</h1>
        <p class="muted small">Bar-by-bar over cached daily price bars. Long-only, single-position, 95% allocation.
            Optimistic fills at signal-bar close — apply your own slippage knob.</p>

        <div class="chart-panel">
            <form id="bt-form" class="inline-form">
                <input name="symbol" placeholder="symbol (AAPL)" value="SPY" required style="text-transform:uppercase">
                <select name="preset" id="ps">
                    ${PRESETS.map(p => `<option value="${p.id}">${esc(p.label)}</option>`).join('')}
                </select>
                <span id="param-slot"></span>
                <label>Days <input name="days" type="number" value="730" style="width:90px"></label>
                <label>Capital <input name="capital" type="number" value="10000" style="width:110px"></label>
                <label>Fee/trade <input name="fee" type="number" step="any" value="1" style="width:80px"></label>
                <button class="primary" type="submit">Run</button>
            </form>
        </div>

        <div id="bt-result"></div>
    `;
    const ps = document.getElementById('ps');
    const slot = document.getElementById('param-slot');
    const renderParams = () => {
        const p = PRESETS.find(x => x.id === ps.value);
        slot.innerHTML = Object.entries(p.defaults).map(([k, v]) =>
            `<label>${k} <input name="${k}" type="number" step="any" value="${v}" style="width:80px"></label>`
        ).join('');
    };
    ps.addEventListener('change', renderParams);
    renderParams();

    document.getElementById('bt-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const preset_id = fd.get('preset');
        const preset_def = PRESETS.find(p => p.id === preset_id);
        const params = {};
        for (const k of Object.keys(preset_def.defaults)) {
            params[k] = Number(fd.get(k));
        }
        const preset = preset_id === 'macd_cross'
            ? 'macd_cross'
            : { [preset_id]: params };
        const body = {
            symbol: fd.get('symbol').trim().toUpperCase(),
            preset,
            days: Number(fd.get('days') || 730),
            initial_capital: Number(fd.get('capital') || 10000),
            fee_per_trade: Number(fd.get('fee') || 0),
        };
        const el = document.getElementById('bt-result');
        el.innerHTML = '<div class="boot">running…</div>';
        try {
            const r = await api.backtestRun(body);
            el.innerHTML = render(r);
            const eqMount = document.getElementById('bt-eq');
            // Adapt to equityChart's expected shape.
            const points = r.equity.map(p => ({
                day: p.time.slice(0, 10),
                cum_net_pnl: p.equity - body.initial_capital,
                drawdown: (p.drawdown_pct / 100) * body.initial_capital,
            }));
            equityChart(eqMount, points, { height: 320 });
        } catch (err) {
            el.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function render(r) {
    const s = r.summary;
    const ret = s.total_return_pct >= 0 ? 'pos' : 'neg';
    return `
        <div class="cards">
            <div class="card"><div class="label">Total return</div>
                <div class="value ${ret}">${s.total_return_pct >= 0 ? '+' : ''}${s.total_return_pct.toFixed(2)}%</div></div>
            <div class="card"><div class="label">Final equity</div><div class="value">$${fmt(s.final_equity)}</div></div>
            <div class="card"><div class="label">Trades</div><div class="value">${s.trades}</div></div>
            <div class="card"><div class="label">Win rate</div><div class="value">${(s.win_rate*100).toFixed(1)}%</div></div>
            <div class="card"><div class="label">Profit factor</div><div class="value">${s.profit_factor.toFixed(2)}</div></div>
            <div class="card"><div class="label">Avg win</div><div class="value pos">$${fmt(s.avg_win)}</div></div>
            <div class="card"><div class="label">Avg loss</div><div class="value neg">$${fmt(s.avg_loss)}</div></div>
            <div class="card"><div class="label">Max DD</div><div class="value neg">${s.max_drawdown_pct.toFixed(2)}%</div></div>
            <div class="card"><div class="label">Sharpe (daily)</div><div class="value">${s.sharpe_daily.toFixed(3)}</div></div>
            <div class="card"><div class="label">Sharpe (ann.)</div><div class="value">${(s.sharpe_daily * Math.sqrt(252)).toFixed(2)}</div></div>
            <div class="card"><div class="label">% time in market</div><div class="value">${s.bars_in_market_pct.toFixed(0)}%</div></div>
        </div>
        <div class="chart-panel">
            <h2>Equity curve</h2>
            <div id="bt-eq"></div>
        </div>
        <div class="chart-panel">
            <h2>Trades · ${r.trades.length}</h2>
            <table class="trades">
                <thead><tr><th>#</th><th>Entry</th><th>Exit</th><th>Bars</th>
                <th>Entry $</th><th>Exit $</th><th>Qty</th><th>P&L</th><th>%</th></tr></thead>
                <tbody>${r.trades.map((t, i) => `
                    <tr>
                        <td>${i+1}</td>
                        <td>${fmtDateTime(t.entry_time)}</td>
                        <td>${fmtDateTime(t.exit_time)}</td>
                        <td>${t.bars_held}</td>
                        <td>${fmt(t.entry_price)}</td>
                        <td>${fmt(t.exit_price)}</td>
                        <td>${fmt(t.qty, 4)}</td>
                        <td class="${t.pnl >= 0 ? 'pos' : 'neg'}">${t.pnl >= 0 ? '+' : ''}$${fmt(t.pnl)}</td>
                        <td class="${t.pnl_pct >= 0 ? 'pos' : 'neg'}">${t.pnl_pct >= 0 ? '+' : ''}${t.pnl_pct.toFixed(2)}%</td>
                    </tr>`).join('')}</tbody>
            </table>
        </div>
    `;
}
