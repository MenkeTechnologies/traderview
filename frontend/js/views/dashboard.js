import { api } from '../api.js';
import { fmt, fmtMoney, fmtPct, fmtSecs, pnlClass, statCard } from '../util.js';
import { equityChart } from '../charts.js';
import { renderWorldMarkets } from './world_map.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderDashboard(mount, state) {
    const tok = currentViewToken();
    if (!state.accountId) {
        mount.innerHTML = `
            <h1 data-i18n="view.dashboard.h1.dashboard" class="view-title">// DASHBOARD</h1>
            <div id="world-markets-mount"></div>
            <p data-i18n="view.dashboard.hint.no_account_yet_add_one_via_accounts_then_import_or" class="boot">No account yet. Add one via Accounts, then import or add trades.</p>
        `;
        const wm = mount.querySelector('#world-markets-mount');
        if (wm) renderWorldMarkets(wm);
        return;
    }
    const [summary, equity, cal] = await Promise.all([
        api.summary(state.accountId),
        api.equity(state.accountId),
        api.calendar(state.accountId),
    ]);
    if (!viewIsCurrent(tok)) return;

    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dashboard.h1.dashboard_2">// DASHBOARD</span>
            <button type="button" class="btn btn-secondary" id="dashboard-refresh-btn"
                    data-i18n="view.dashboard.btn.refresh"
                    data-tip="view.dashboard.tip.refresh"
                    data-shortcut="dashboard_refresh"
                    style="margin-left:12px;font-size:11px;padding:4px 10px;vertical-align:middle">⟳ Refresh</button>
        </h1>
        <div id="world-markets-mount"></div>
        <div class="cards">
            ${statCard(t('view.dashboard.stat.net_pnl'),      fmtMoney(summary.net_pnl), pnlClass(summary.net_pnl))}
            ${statCard(t('view.dashboard.stat.trades'),       summary.trade_count)}
            ${statCard(t('view.dashboard.stat.win_rate'),     fmtPct(summary.win_rate))}
            ${statCard(t('view.dashboard.stat.profit_factor'), fmt(summary.profit_factor))}
            ${statCard(t('view.dashboard.stat.expectancy'),   fmtMoney(summary.expectancy), pnlClass(summary.expectancy))}
            ${statCard(t('view.dashboard.stat.avg_r'),        fmt(summary.avg_r))}
            ${statCard(t('view.dashboard.stat.largest_win'),  fmtMoney(summary.largest_win), 'pos')}
            ${statCard(t('view.dashboard.stat.largest_loss'), fmtMoney(summary.largest_loss), 'neg')}
            ${statCard(t('view.dashboard.stat.max_consec_wins'),   summary.max_consec_wins)}
            ${statCard(t('view.dashboard.stat.max_consec_losses'), summary.max_consec_losses)}
            ${statCard(t('view.dashboard.stat.avg_hold'),     fmtSecs(summary.avg_hold_seconds))}
            ${statCard(t('view.dashboard.stat.fees'),         fmtMoney(summary.fees))}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.equity_curve">Equity Curve</h2>
            <div id="equity-chart"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.last_90_days">Last 90 Days</h2>
            <div class="mini-cal" id="mini-cal"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.daily_pnl_chart">Daily P&L (last 90 trading days)</h2>
            <div id="dash-pnl-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.trades_chart">Trade count per day (last 90)</h2>
            <div id="dash-tr-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.risk_gate_today">🛡 Risk Gate · today</h2>
            <div id="dash-rg" class="muted small" data-i18n="common.loading">loading…</div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dashboard.h2.discipline_score_last_7_days">📐 Discipline score · last 7 days</h2>
            <div id="dash-disc" data-i18n="common.loading">loading…</div>
        </div>
    `;

    const refreshBtn = mount.querySelector('#dashboard-refresh-btn');
    if (refreshBtn) refreshBtn.addEventListener('click', () =>
        window.dispatchEvent(new HashChangeEvent('hashchange')));
    const eqEl = mount.querySelector('#equity-chart');
    const calEl = mount.querySelector('#mini-cal');
    const wmEl = mount.querySelector('#world-markets-mount');
    const rgEl = mount.querySelector('#dash-rg');
    const discEl = mount.querySelector('#dash-disc');
    if (eqEl) equityChart(eqEl, equity);
    if (calEl) renderMiniCalendar(calEl, cal);
    renderDailyPnlChart(cal);
    renderTradesChart(cal);
    if (wmEl) renderWorldMarkets(wmEl);
    if (rgEl) loadRiskGateBadge(rgEl);
    if (discEl && state.accountId) loadDisciplineScore(discEl, state.accountId);
}

async function loadDisciplineScore(el, accountId) {
    try {
        const s = await api.disciplineScore(accountId, 7);
        const color = s.score >= 90 ? '#39ff14'
                    : s.score >= 75 ? '#ffb800'
                                    : '#ff2a6d';
        const body = t('view.dashboard.discipline.body', {
            stop_set:       s.component_stop_set,
            stop_honored:   s.component_stop_honored,
            plan:           s.component_plan_adherence,
            gate_restraint: s.component_gate_restraint,
        });
        const win = t('view.dashboard.discipline.window', {
            blocks:        s.gate_blocks,
            block_label:   t(s.gate_blocks === 1 ? 'view.dashboard.discipline.block_singular' : 'view.dashboard.discipline.block_plural'),
            warnings:      s.gate_warnings,
            warning_label: t(s.gate_warnings === 1 ? 'view.dashboard.discipline.warning_singular' : 'view.dashboard.discipline.warning_plural'),
        });
        el.innerHTML = `
            <div style="display:flex;align-items:center;gap:20px;flex-wrap:wrap">
                <div style="font-size:48px;font-weight:700;color:${color};line-height:1">${s.score}</div>
                <div style="font-size:24px;color:${color}">${esc(s.grade)}</div>
                <div class="muted small" style="flex:1;min-width:200px">
                    ${esc(body)}
                    <br>${esc(win)}
                </div>
            </div>
        `;
    } catch (_) {
        el.textContent = t('view.dashboard.discipline.unavailable');
    }
}

function esc(s) { return String(s).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c])); }

async function loadRiskGateBadge(el) {
    try {
        const fires = await api.riskFires(200);
        const today = new Date().toISOString().slice(0, 10);
        const todays = fires.filter(f => f.fired_at.slice(0, 10) === today);
        const blocks = todays.filter(f => f.blocked).length;
        const warns  = todays.length - blocks;
        if (!todays.length) {
            el.innerHTML = `<span class="muted">${t('view.dashboard.empty.no_fires_today')}</span>`;
            return;
        }
        el.innerHTML = t('view.dashboard.risk_gate.body', {
            blocks_html: `<strong style="color:#ff2a6d">${blocks}</strong>`,
            warns_html:  `<strong style="color:#ffb800">${warns}</strong>`,
            audit_link:  `<a href="#risk-gate">${esc(t('view.dashboard.risk_gate.audit_log'))}</a>`,
        });
    } catch (_) {
        el.textContent = t('view.dashboard.risk_gate.unavailable');
    }
}

function renderTradesChart(cells) {
    const el = document.getElementById('dash-tr-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (cells || [])
        .filter(c => c.day && Number.isFinite(Number(c.trades)))
        .slice(-90);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.dashboard.empty_tr_chart">${esc(t('view.dashboard.empty_tr_chart'))}</div>`;
        return;
    }
    const labels = rows.map(c => c.day);
    const xs = labels.map((_, i) => i + 1);
    const ys = rows.map(c => Number(c.trades));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.dashboard.chart.day') },
            { label: t('view.dashboard.chart.trades'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderDailyPnlChart(cells) {
    const el = document.getElementById('dash-pnl-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (cells || [])
        .filter(c => c.day && Number.isFinite(Number(c.net_pnl)))
        .slice(-90);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.dashboard.empty_chart">${esc(t('view.dashboard.empty_chart'))}</div>`;
        return;
    }
    const labels = rows.map(c => c.day);
    const xs = labels.map((_, i) => i + 1);
    const winY  = rows.map(c => Number(c.net_pnl) >= 0 ? Number(c.net_pnl) : null);
    const loseY = rows.map(c => Number(c.net_pnl) <  0 ? Number(c.net_pnl) : null);
    const zero  = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.dashboard.chart.day') },
            { label: t('view.dashboard.chart.win'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.dashboard.chart.lose'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 10, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.dashboard.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, winY, loseY, zero], el);
}

function renderMiniCalendar(el, cells) {
    if (!cells.length) { el.innerHTML = `<div class="boot">${t('view.dashboard.empty.no_data')}</div>`; return; }
    const recent = cells.slice(-90);
    const max = Math.max(...recent.map(c => Math.abs(Number(c.net_pnl))), 1);
    el.innerHTML = recent.map(c => {
        const v = Number(c.net_pnl);
        const intensity = Math.min(1, Math.abs(v) / max);
        const color = v >= 0
            ? `rgba(35, 209, 96, ${0.15 + intensity * 0.7})`
            : `rgba(255, 56, 96, ${0.15 + intensity * 0.7})`;
        return `<div class="cal-cell" style="background:${color}"
            title="${esc(t('view.dashboard.cal.tooltip', { day: c.day, pnl: fmtMoney(v), n: c.trades }))}"></div>`;
    }).join('');
}
