// Side-by-side broker comparison.
//
// Mirrors business_compare.js but for trading: one column per broker
// (plus an aggregated column) showing the same KPI strip (net P&L,
// win rate, profit factor, avg win/loss, trade count, fees, max
// drawdown, biggest gain/loss, expectancy) so the user can scan
// brokers without switching tabs.
//
// Backed by parallel /api/stats/summary calls, one per broker.

import { api } from '../api.js';
import { fmtMoney } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { listBrokers } from '../broker_context.js';

const PERIOD_KEY = 'broker_compare_days';

function esc(s) {
    return String(s).replace(/[&<>"]/g, (c) => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;',
    }[c]));
}

function getDays() {
    const v = Number(localStorage.getItem(PERIOD_KEY));
    return [30, 60, 90, 365].includes(v) ? v : 90;
}
function setDays(d) {
    if ([30, 60, 90, 365].includes(d)) localStorage.setItem(PERIOD_KEY, String(d));
}

// Issue a summary fetch without the global broker filter intercepting —
// the global rq() function in api.js would otherwise overwrite our
// explicit broker_id with whichever broker the user has selected.
// `summaryRaw` routes through the same `request()` helper as the rest
// of api.js (so Tauri's `baseUrl` + bearer header are wired up) but
// skips rq() entirely, sending only the params we hand it.
async function summaryForBroker(brokerId, days) {
    const params = { days };
    if (brokerId) params.broker_id = brokerId;
    return api.summaryRaw(params);
}

export async function renderBrokerCompare(mount) {
    const tok = currentViewToken();
    let brokers = [];
    try { brokers = await listBrokers(); } catch { brokers = []; }
    if (!viewIsCurrent(tok)) return;

    const days = getDays();
    const dayOpts = [30, 60, 90, 365].map(d =>
        `<option value="${d}"${d === days ? ' selected' : ''}>${d}d</option>`).join('');

    mount.innerHTML = `
        <div class="bizc-header">
            <h1 class="view-title">
                <span data-i18n="view.broker_compare.title">// BROKER COMPARISON</span>
            </h1>
            <div class="bizc-controls">
                <label>${esc(t('view.broker_compare.window'))}
                    <select id="bkrc-days">${dayOpts}</select></label>
            </div>
        </div>
        <div id="bkrc-grid" class="bizc-grid">
            <div class="muted small">${esc(t('common.loading'))}</div>
        </div>
    `;
    mount.querySelector('#bkrc-days').addEventListener('change', (e) => {
        setDays(Number(e.target.value));
        renderBrokerCompare(mount);
    });
    if (brokers.length === 0) {
        mount.querySelector('#bkrc-grid').innerHTML = `
            <div class="bizc-empty">
                <p>${esc(t('view.broker_compare.empty'))}</p>
                <p class="muted small">${esc(t('view.broker_compare.empty_hint'))}</p>
            </div>`;
        return;
    }

    // Fetch one summary per broker + one aggregated. Parallel.
    let bundles;
    try {
        bundles = await Promise.all([
            summaryForBroker(null, days),
            ...brokers.map(b => summaryForBroker(b.id, days)),
        ]);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.broker_compare.load_failed', { err: e.message }), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const columns = [
        { label: t('view.broker_compare.aggregated'), key: 'all', summary: bundles[0] },
        ...brokers.map((b, i) => ({
            label: b.display_name,
            key: b.id,
            summary: bundles[i + 1],
        })),
    ];

    const kpis = [
        { key: 'net_pnl', label: t('view.broker_compare.kpi.net_pnl'), fmt: fmtMoney, higherBetter: true },
        { key: 'win_rate', label: t('view.broker_compare.kpi.win_rate'),
            fmt: (v) => `${((+v || 0) * 100).toFixed(1)}%`, higherBetter: true },
        { key: 'profit_factor', label: t('view.broker_compare.kpi.profit_factor'),
            fmt: (v) => Number.isFinite(+v) ? (+v).toFixed(2) : '—', higherBetter: true },
        { key: 'expectancy', label: t('view.broker_compare.kpi.expectancy'), fmt: fmtMoney, higherBetter: true },
        { key: 'avg_win', label: t('view.broker_compare.kpi.avg_win'), fmt: fmtMoney, higherBetter: true },
        { key: 'avg_loss', label: t('view.broker_compare.kpi.avg_loss'), fmt: fmtMoney, higherBetter: true },
        { key: 'largest_win', label: t('view.broker_compare.kpi.largest_win'), fmt: fmtMoney, higherBetter: true },
        { key: 'largest_loss', label: t('view.broker_compare.kpi.largest_loss'), fmt: fmtMoney, higherBetter: true },
        { key: 'fees', label: t('view.broker_compare.kpi.fees'), fmt: fmtMoney, higherBetter: false },
        { key: 'trade_count', label: t('view.broker_compare.kpi.trade_count'),
            fmt: (v) => String(v ?? 0) },
        { key: 'avg_hold_time_secs', label: t('view.broker_compare.kpi.avg_hold'),
            fmt: (v) => `${Math.round((+v || 0) / 60)}m` },
        { key: 'max_consec_losses', label: t('view.broker_compare.kpi.max_consec_losses'),
            fmt: (v) => String(v ?? 0), higherBetter: false },
    ];

    const headerRow = `<thead><tr>
        <th class="bizc-kpi-label"></th>
        ${columns.map(c => `<th class="bizc-col-head" data-key="${esc(c.key)}">${esc(c.label)}</th>`).join('')}
    </tr></thead>`;

    const bodyRows = kpis.map(k => {
        const vals = columns.map(c => Number(c.summary?.[k.key] ?? 0));
        const perBroker = vals.slice(1);
        let bestIdx = -1;
        if (perBroker.length > 0 && k.higherBetter !== undefined) {
            const ext = k.higherBetter
                ? Math.max(...perBroker)
                : Math.min(...perBroker);
            if (Number.isFinite(ext)) bestIdx = perBroker.indexOf(ext) + 1;
        }
        return `<tr>
            <th class="bizc-kpi-label">${esc(k.label)}</th>
            ${columns.map((c, i) => {
                const v = vals[i];
                const cls = i === bestIdx ? 'bizc-best' : (i === 0 ? 'bizc-agg' : '');
                return `<td class="${cls}"><strong>${esc(k.fmt(v))}</strong></td>`;
            }).join('')}
        </tr>`;
    }).join('');

    mount.querySelector('#bkrc-grid').innerHTML = `
        <table class="bizc-table">
            ${headerRow}
            <tbody>${bodyRows}</tbody>
        </table>
        <p class="muted small bizc-footer">
            ${esc(t('view.broker_compare.footer_legend'))}
        </p>
    `;
}
