// Live Dashboard — full chart set powered by broker REST data.
// Pulls /v2/account, /v2/positions, /v2/orders, /v2/account/portfolio/history
// from Alpaca every 15 s. Derives daily P/L series, calendar cells,
// drawdown, and per-position breakdowns without touching the local
// executions/trades pipeline.

import { api } from '../api.js';
import { equityChart, barChart } from '../charts.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const REFRESH_MS = 15000;

export async function renderLiveDashboard(mount, state) {
    const accountId = state?.accountId;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.live_dashboard.title">// LIVE DASHBOARD ●</span></h1>
        <p class="muted small" data-i18n="view.live_dashboard.intro">
            Live broker state pulled directly from Alpaca REST every 15 seconds.
            Equity, buying power, open positions, recent orders, equity curve,
            calendar, daily P&L — straight from the source. No CSV, no closed-trade
            round-trip required.
        </p>
        ${!accountId ? `
            <div class="chart-panel">
                <p class="muted">${esc(t('view.live_dashboard.no_account') || 'Pick an account from the header dropdown to load live data.')}</p>
            </div>
        ` : `
            <div style="display:flex;gap:8px;align-items:center;margin-bottom:8px;flex-wrap:wrap">
                <button class="btn btn-sm primary" id="ld-refresh" data-tip="Force-refresh from broker now">↻ Refresh</button>
                <label style="display:flex;gap:4px;align-items:center">
                    <input type="checkbox" id="ld-auto" checked>
                    <span class="muted small">Auto every ${REFRESH_MS / 1000}s</span>
                </label>
                <label style="display:flex;gap:4px;align-items:center">
                    <span class="muted small">History window</span>
                    <select id="ld-window" style="width:90px">
                        <option value="1W">1W</option>
                        <option value="1M" selected>1M</option>
                        <option value="3M">3M</option>
                        <option value="1A">1Y</option>
                        <option value="all">All</option>
                    </select>
                </label>
                <span id="ld-status" class="muted small"></span>
            </div>
            <div id="ld-banner"></div>
            <div id="ld-account" class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px"></div>

            <h3 class="section-title">Equity Curve</h3>
            <div class="chart-panel">
                <div id="ld-equity" class="chart-h-280"></div>
            </div>

            <h3 class="section-title" style="margin-top:18px">Drawdown</h3>
            <div class="chart-panel">
                <div id="ld-drawdown" class="chart-h-200"></div>
            </div>

            <h3 class="section-title" style="margin-top:18px">Daily P&L</h3>
            <div class="chart-panel">
                <div id="ld-daily" class="chart-h-200"></div>
            </div>

            <h3 class="section-title" style="margin-top:18px">Calendar — Daily P&L</h3>
            <div class="chart-panel">
                <div id="ld-calendar"></div>
            </div>

            <h3 class="section-title" style="margin-top:18px">Position P&L Breakdown</h3>
            <div class="chart-panel">
                <div id="ld-pos-bars" class="chart-h-200"></div>
            </div>

            <h3 class="section-title" style="margin-top:18px">Allocation by Symbol</h3>
            <div class="chart-panel">
                <div id="ld-alloc"></div>
            </div>

            <h3 class="section-title" style="margin-top:18px">Open Positions</h3>
            <div id="ld-positions" class="chart-panel"></div>

            <h3 class="section-title" style="margin-top:18px">Recent Orders (last 100)</h3>
            <div id="ld-orders" class="chart-panel"></div>
        `}
    `;
    if (!accountId) return;

    let timer = null;
    let stopped = false;
    let currentWindow = '1M';

    async function load() {
        if (stopped) return;
        const status = mount.querySelector('#ld-status');
        if (status) status.textContent = 'fetching…';
        try {
            const url = `/live/dashboard?account_id=${encodeURIComponent(accountId)}&window=${currentWindow}`;
            const data = await api.request(url);
            paint(mount, data);
            if (status) status.textContent = `updated ${new Date().toLocaleTimeString()}`;
        } catch (e) {
            if (status) status.textContent = `error: ${e?.message || e}`;
            const banner = mount.querySelector('#ld-banner');
            if (banner) banner.innerHTML = `<p class="neg small">Fetch failed: ${esc(String(e?.message || e))}</p>`;
        }
    }

    function scheduleNext() {
        clearTimeout(timer);
        if (stopped) return;
        const auto = mount.querySelector('#ld-auto');
        if (auto && auto.checked) {
            timer = setTimeout(async () => { await load(); scheduleNext(); }, REFRESH_MS);
        }
    }

    mount.querySelector('#ld-refresh').addEventListener('click', async () => { await load(); scheduleNext(); });
    mount.querySelector('#ld-auto').addEventListener('change', scheduleNext);
    mount.querySelector('#ld-window').addEventListener('change', async (ev) => {
        currentWindow = ev.target.value;
        await load();
        scheduleNext();
    });

    await load();
    scheduleNext();

    return () => { stopped = true; clearTimeout(timer); };
}

function paint(mount, data) {
    const banner = mount.querySelector('#ld-banner');
    if (!data.connected) {
        banner.innerHTML = `<div class="chart-panel"><p class="neg">${esc(data.note || 'Not connected')}</p></div>`;
    } else if (data.note) {
        banner.innerHTML = `<p class="muted small">⚠ ${esc(data.note)}</p>`;
    } else {
        banner.innerHTML = '';
    }

    paintAccount(mount, data);
    paintHistoryCharts(mount, data);
    paintCalendar(mount, data);
    paintPositionBreakdown(mount, data);
    paintAllocation(mount, data);
    paintPositionsTable(mount, data);
    paintOrdersTable(mount, data);
}

function paintAccount(mount, data) {
    const accountEl = mount.querySelector('#ld-account');
    const a = data.account;
    if (!a) {
        accountEl.innerHTML = `<p class="muted small">No account snapshot.</p>`;
        return;
    }
    const portfolioValue = num(a.portfolio_value);
    const equity = num(a.equity);
    const cash = num(a.cash);
    const buyingPower = num(a.buying_power);
    const dayTradeCount = a.daytrade_count ?? 0;
    const pdt = a.pattern_day_trader ? 'PDT' : '';
    // Day change from portfolio_history. Filter to real (non-null, non-zero)
    // equity values so a pre-funding zero doesn't make the delta look like a
    // huge negative number.
    let dayChange = null;
    if (data.history && data.history.equity) {
        const eq = data.history.equity.filter(v => v != null && v > 0);
        if (eq.length >= 2) {
            const last = eq[eq.length - 1];
            const prev = eq[eq.length - 2];
            dayChange = last - prev;
        }
    }
    const dayChangeEl = dayChange == null ? '—' :
        `<span class="${dayChange >= 0 ? 'pos' : 'neg'}">${dayChange >= 0 ? '+' : ''}$${fmt(dayChange, 2)}</span>`;
    accountEl.innerHTML = `
        <div class="card"><div class="label">Portfolio value</div><div class="value">$${fmt(portfolioValue, 2)}</div><div class="muted small">${esc(data.mode)} · ${esc(a.status || '')}</div></div>
        <div class="card"><div class="label">Equity</div><div class="value">$${fmt(equity, 2)}</div></div>
        <div class="card"><div class="label">Day change</div><div class="value">${dayChangeEl}</div></div>
        <div class="card"><div class="label">Cash</div><div class="value">$${fmt(cash, 2)}</div></div>
        <div class="card"><div class="label">Buying power</div><div class="value pos">$${fmt(buyingPower, 2)}</div></div>
        <div class="card"><div class="label">Open positions</div><div class="value">${data.position_count}</div></div>
        <div class="card"><div class="label">Unrealized P&L</div><div class="value ${data.total_unrealized_pl >= 0 ? 'pos' : 'neg'}">${data.total_unrealized_pl >= 0 ? '+' : ''}$${fmt(data.total_unrealized_pl, 2)}</div><div class="muted small">Market value $${fmt(data.total_market_value, 0)}</div></div>
        <div class="card"><div class="label">Day-trade count</div><div class="value">${dayTradeCount} ${pdt ? `<span class="neg small">${esc(pdt)}</span>` : ''}</div></div>
    `;
}

function paintHistoryCharts(mount, data) {
    const eqEl = mount.querySelector('#ld-equity');
    const ddEl = mount.querySelector('#ld-drawdown');
    const dailyEl = mount.querySelector('#ld-daily');
    const h = data.history;
    if (!h || !h.timestamp || !h.timestamp.length) {
        const empty = `<p class="muted small">No portfolio history available.</p>`;
        if (eqEl) eqEl.innerHTML = empty;
        if (ddEl) ddEl.innerHTML = empty;
        if (dailyEl) dailyEl.innerHTML = empty;
        return;
    }
    // Alpaca's portfolio_history returns equity=0 for days BEFORE the account
    // was funded — using base_value as the cum baseline turns those into a
    // fake -base_value drawdown cliff. Trim leading zero/null entries and
    // use the first real equity as the "you started here" baseline.
    let firstIdx = -1;
    for (let i = 0; i < h.timestamp.length; i++) {
        const eq = h.equity[i];
        if (eq != null && eq > 0) { firstIdx = i; break; }
    }
    if (firstIdx < 0) {
        const empty = `<p class="muted small">Account hasn't been funded yet — no equity history.</p>`;
        if (eqEl) eqEl.innerHTML = empty;
        if (ddEl) ddEl.innerHTML = empty;
        if (dailyEl) dailyEl.innerHTML = empty;
        return;
    }
    const startingEquity = h.equity[firstIdx];
    const points = [];
    let peak = 0;
    let prev = null;
    for (let i = firstIdx; i < h.timestamp.length; i++) {
        const eq = h.equity[i];
        if (eq == null) continue;
        const cum = eq - startingEquity;
        if (cum > peak) peak = cum;
        const drawdown = cum - peak;
        const dayPl = prev == null ? 0 : (eq - prev);
        prev = eq;
        points.push({
            day: new Date(h.timestamp[i] * 1000).toISOString().slice(0, 10),
            cum_net_pnl: cum,
            day_net_pnl: dayPl,
            drawdown,
        });
    }

    if (eqEl && window.uPlot) equityChart(eqEl, points);

    // Drawdown standalone bar/line.
    if (ddEl && window.uPlot && points.length) {
        const labels = points.map(p => p.day);
        const values = points.map(p => p.drawdown);
        barChart(ddEl, labels, values, { color: '#ff3860', yKind: 'money' });
    }

    // Daily P&L bars.
    if (dailyEl && window.uPlot && points.length) {
        const labels = points.map(p => p.day);
        const values = points.map(p => p.day_net_pnl);
        barChart(dailyEl, labels, values, { color: '#00e5ff', yKind: 'money' });
    }
}

function paintCalendar(mount, data) {
    const el = mount.querySelector('#ld-calendar');
    if (!el) return;
    const h = data.history;
    if (!h || !h.timestamp || !h.timestamp.length) {
        el.innerHTML = `<p class="muted small">No data for calendar.</p>`;
        return;
    }
    // Skip pre-funding zeros same as the equity-curve loop.
    let firstIdx = -1;
    for (let i = 0; i < h.timestamp.length; i++) {
        const eq = h.equity[i];
        if (eq != null && eq > 0) { firstIdx = i; break; }
    }
    if (firstIdx < 0) {
        el.innerHTML = `<p class="muted small">Account hasn't been funded yet — no calendar data.</p>`;
        return;
    }
    const byMonth = new Map();
    let prev = null;
    for (let i = firstIdx; i < h.timestamp.length; i++) {
        const eq = h.equity[i];
        if (eq == null) continue;
        const dayPl = prev == null ? 0 : (eq - prev);
        prev = eq;
        const d = new Date(h.timestamp[i] * 1000);
        const monthKey = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
        if (!byMonth.has(monthKey)) byMonth.set(monthKey, []);
        byMonth.get(monthKey).push({ d, dayPl, eq });
    }
    if (!byMonth.size) {
        el.innerHTML = `<p class="muted small">No data for calendar.</p>`;
        return;
    }
    const monthEntries = Array.from(byMonth.entries()).sort();
    el.innerHTML = monthEntries.map(([monthKey, days]) => {
        const monthLabel = new Date(days[0].d.getFullYear(), days[0].d.getMonth(), 1)
            .toLocaleString(undefined, { month: 'long', year: 'numeric' });
        const dayCount = days.length;
        const monthPl = days.reduce((s, x) => s + x.dayPl, 0);
        const winners = days.filter(x => x.dayPl > 0).length;
        const losers = days.filter(x => x.dayPl < 0).length;
        const cells = days.map(x => {
            const cls = x.dayPl > 0 ? 'pos' : x.dayPl < 0 ? 'neg' : 'muted';
            const dd = String(x.d.getDate()).padStart(2, '0');
            return `<div class="cal-cell ${cls}" title="${dd}: ${x.dayPl >= 0 ? '+' : ''}$${fmt(x.dayPl, 2)} · equity $${fmt(x.eq, 0)}">
                <div class="cal-day">${dd}</div>
                <div class="cal-pl">${x.dayPl >= 0 ? '+' : ''}$${fmt(x.dayPl, 0)}</div>
            </div>`;
        }).join('');
        return `<div style="margin-bottom:12px">
            <div style="display:flex;gap:10px;align-items:baseline;margin-bottom:6px">
                <strong>${esc(monthLabel)}</strong>
                <span class="muted small">${dayCount} day${dayCount === 1 ? '' : 's'}</span>
                <span class="${monthPl >= 0 ? 'pos' : 'neg'} small"><strong>${monthPl >= 0 ? '+' : ''}$${fmt(monthPl, 2)}</strong></span>
                <span class="pos small">${winners}W</span>
                <span class="neg small">${losers}L</span>
            </div>
            <div class="cal-grid" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(72px,1fr));gap:4px">${cells}</div>
        </div>`;
    }).join('');
}

function paintPositionBreakdown(mount, data) {
    const el = mount.querySelector('#ld-pos-bars');
    if (!el) return;
    if (!data.positions || !data.positions.length) {
        el.innerHTML = `<p class="muted small">No open positions to chart.</p>`;
        return;
    }
    const sorted = data.positions.slice().sort((a, b) => num(b.unrealized_pl) - num(a.unrealized_pl));
    const labels = sorted.map(p => p.symbol);
    const values = sorted.map(p => num(p.unrealized_pl));
    if (window.uPlot) barChart(el, labels, values, { color: '#00e5ff', yKind: 'money' });
}

function paintAllocation(mount, data) {
    const el = mount.querySelector('#ld-alloc');
    if (!el) return;
    if (!data.positions || !data.positions.length) {
        el.innerHTML = `<p class="muted small">No positions.</p>`;
        return;
    }
    const totalMv = data.positions.reduce((s, p) => s + Math.abs(num(p.market_value)), 0);
    if (totalMv === 0) {
        el.innerHTML = `<p class="muted small">All positions have zero market value.</p>`;
        return;
    }
    const rows = data.positions.slice().sort((a, b) => Math.abs(num(b.market_value)) - Math.abs(num(a.market_value)));
    el.innerHTML = `
        <div style="display:grid;grid-template-columns:1fr;gap:4px">
            ${rows.map(p => {
                const mv = Math.abs(num(p.market_value));
                const pct = (mv / totalMv) * 100;
                const upl = num(p.unrealized_pl);
                return `<div style="display:grid;grid-template-columns:80px 1fr 100px 90px;gap:8px;align-items:center">
                    <div><strong>${esc(p.symbol)}</strong></div>
                    <div style="background:#1a1a22;border-radius:3px;height:18px;position:relative;overflow:hidden">
                        <div style="background:${p.side === 'long' ? '#00e5ff' : '#ff8b3d'};height:100%;width:${pct}%"></div>
                    </div>
                    <div class="muted small">$${fmt(mv, 0)}</div>
                    <div class="${upl >= 0 ? 'pos' : 'neg'}">${upl >= 0 ? '+' : ''}$${fmt(upl, 0)}</div>
                </div>`;
            }).join('')}
        </div>
        <p class="muted small" style="margin-top:6px">Total market value: $${fmt(totalMv, 0)}</p>
    `;
}

function paintPositionsTable(mount, data) {
    const posEl = mount.querySelector('#ld-positions');
    if (!posEl) return;
    if (!data.positions.length) {
        posEl.innerHTML = `<p class="muted small">No open positions.</p>`;
        return;
    }
    posEl.innerHTML = `
        <table class="trades" data-table-key="ld-pos">
            <thead><tr>
                <th>Symbol</th>
                <th>Side</th>
                <th>Qty</th>
                <th>Avg Entry</th>
                <th>Current</th>
                <th>Market Value</th>
                <th>Unrealized P&L</th>
                <th>%</th>
            </tr></thead>
            <tbody>${data.positions.map(p => {
                const q = num(p.qty);
                const avg = num(p.avg_entry_price);
                const cur = num(p.current_price);
                const mv = num(p.market_value);
                const up = num(p.unrealized_pl);
                const upPct = avg > 0 ? ((cur - avg) / avg * 100) * (p.side === 'short' ? -1 : 1) : 0;
                return `<tr>
                    <td><strong>${esc(p.symbol)}</strong></td>
                    <td class="${p.side === 'long' ? 'pos' : 'neg'}">${esc(p.side.toUpperCase())}</td>
                    <td>${fmt(q, 4)}</td>
                    <td>$${fmt(avg, 2)}</td>
                    <td>$${fmt(cur, 2)}</td>
                    <td>$${fmt(mv, 2)}</td>
                    <td class="${up >= 0 ? 'pos' : 'neg'}"><strong>${up >= 0 ? '+' : ''}$${fmt(up, 2)}</strong></td>
                    <td class="${upPct >= 0 ? 'pos' : 'neg'}">${upPct >= 0 ? '+' : ''}${fmt(upPct, 2)}%</td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
}

function paintOrdersTable(mount, data) {
    const ordEl = mount.querySelector('#ld-orders');
    if (!ordEl) return;
    if (!data.orders.length) {
        ordEl.innerHTML = `<p class="muted small">No recent orders.</p>`;
        return;
    }
    ordEl.innerHTML = `
        <table class="trades" data-table-key="ld-ord">
            <thead><tr>
                <th>Submitted</th>
                <th>Symbol</th>
                <th>Side</th>
                <th>Type</th>
                <th>Qty</th>
                <th>Filled</th>
                <th>Avg Fill</th>
                <th>Status</th>
            </tr></thead>
            <tbody>${data.orders.slice(0, 100).map(o => {
                const created = o.created_at ? new Date(o.created_at).toLocaleString() : '—';
                const qty = num(o.qty);
                const fq = num(o.filled_qty);
                const fp = num(o.filled_avg_price);
                const statusCls = o.status === 'filled' ? 'pos' : (o.status === 'canceled' || o.status === 'rejected' || o.status === 'expired') ? 'neg' : 'muted';
                return `<tr>
                    <td class="muted small">${esc(created)}</td>
                    <td><strong>${esc(o.symbol)}</strong></td>
                    <td class="${o.side === 'buy' ? 'pos' : 'neg'}">${esc(o.side.toUpperCase())}</td>
                    <td class="muted">${esc(o.order_type)}</td>
                    <td>${fmt(qty, 4)}</td>
                    <td>${fmt(fq, 4)}</td>
                    <td>${fp > 0 ? '$' + fmt(fp, 2) : '—'}</td>
                    <td class="${statusCls}">${esc(o.status)}</td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
}

function num(v) {
    if (v == null) return 0;
    const n = typeof v === 'string' ? parseFloat(v) : Number(v);
    return Number.isFinite(n) ? n : 0;
}

function fmt(n, d) {
    if (n == null || !Number.isFinite(Number(n))) return '—';
    return Number(n).toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
}
