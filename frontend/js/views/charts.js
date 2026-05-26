import { api } from '../api.js';
import { ohlcChart } from '../charts.js';
import { esc } from '../util.js';

export async function renderCharts(mount, _state, symbol = '') {
    if (!symbol) symbol = 'SPY';
    mount.innerHTML = `
        <h1 class="view-title">// CHARTS</h1>
        <div class="chart-toolbar">
            <label>Symbol <input id="sym" value="${esc(symbol)}"></label>
            <label>Interval
                <select id="iv">
                    <option value="1m">1m</option>
                    <option value="5m">5m</option>
                    <option value="15m">15m</option>
                    <option value="1h">1h</option>
                    <option value="1d" selected>1d</option>
                    <option value="1w">1w</option>
                </select>
            </label>
            <label>From <input type="date" id="from"></label>
            <label>To <input type="date" id="to"></label>
            <button class="primary" id="load">Load</button>
        </div>
        <div class="chart-panel"><div id="chart-mount"></div></div>
    `;

    const to = new Date();
    const from = new Date(to.getTime() - 90 * 86400_000);
    document.getElementById('from').value = from.toISOString().slice(0, 10);
    document.getElementById('to').value = to.toISOString().slice(0, 10);

    const load = async () => {
        const sym = document.getElementById('sym').value.trim();
        const iv = document.getElementById('iv').value;
        const f = Math.floor(new Date(document.getElementById('from').value).getTime() / 1000);
        const t = Math.floor(new Date(document.getElementById('to').value).getTime() / 1000) + 86400;
        try {
            const resp = await api.bars(sym, iv, f, t);
            ohlcChart(document.getElementById('chart-mount'), resp.bars, [], { height: 480 });
        } catch (e) {
            document.getElementById('chart-mount').innerHTML =
                `<div class="boot">Error: ${e.message}</div>`;
        }
    };

    document.getElementById('load').addEventListener('click', load);
    load();
}
