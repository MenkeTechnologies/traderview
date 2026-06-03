import { api } from '../api.js';
import { fmtDateTime, fmtMoney, fmtSecs, md, esc, pnlClass } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderJournalView(mount, state, dayOrGeneral) {
    const tok = currentViewToken();
    const isGeneral = dayOrGeneral === 'general';
    const day = isGeneral ? null : (dayOrGeneral || new Date().toISOString().slice(0, 10));
    const acct = state?.accountId;

    // Fetch entries + (for day view) the day's trades in parallel. The trades
    // power a Tradervue-style "day summary" card at the top: net P&L, trade
    // count, win/loss splits, total volume, etc.
    const [entries, dayTrades] = await Promise.all([
        isGeneral ? api.journalGeneral() : api.journalForDay(day),
        (!isGeneral && acct)
            ? api.trades(acct, { date_from: day, date_to: day, limit: 200 }).catch(() => [])
            : Promise.resolve([]),
    ]);
    if (!viewIsCurrent(tok)) return;

    mount.innerHTML = `
        <h1 class="view-title">
            // JOURNAL ·
            ${isGeneral
                ? `<span class="journal-label-magenta">${esc(t('view.journal.label.general'))}</span>`
                : `<input type="date" id="journal-day" value="${day}" data-tip="view.journal.tip.day">`}
            <a href="#journal/${isGeneral ? new Date().toISOString().slice(0,10) : 'general'}" class="link small">
                ${esc(t(isGeneral ? 'view.journal.link.switch_to_daily' : 'view.journal.link.switch_to_general'))}
            </a>
            <button type="button" class="btn btn-secondary" id="journal-refresh-btn"
                    data-i18n="view.journal.btn.refresh"
                    data-tip="view.journal.tip.refresh"
                    data-shortcut="journal_refresh"
                    class="btn btn-secondary journal-header-refresh">⟳ Refresh</button>
        </h1>
        ${isGeneral ? '' : dayJourneySummary(day, dayTrades)}
        <div id="entries">${entries.map(e => `
            <div class="journal-entry"
                 data-context-scope="journal-entry"
                 data-id="${esc(e.id)}"
                 data-trade-id="${esc(e.trade_id || '')}">
                <div class="meta">
                    ${fmtDateTime(e.created_at)}
                    ${e.mood !== null ? `· mood ${e.mood}` : ''}
                    ${e.trade_id ? `· <a href="#trade/${e.trade_id}">${esc(t('common.link.trade'))}</a>` : ''}
                </div>
                <div class="body">${md(e.body_md)}</div>
                <button data-i18n="view.journal.btn.delete" class="link" data-del="${e.id}">delete</button>
            </div>
        `).join('') || `<p class="muted">${esc(t(isGeneral ? 'view.journal.empty.general' : 'view.journal.empty.day'))}</p>`}</div>
        <div class="chart-panel">
            <h2 data-i18n="view.journal.h2.mood_chart">Mood trend (per entry, -2..+2)</h2>
            <div id="j-chart" class="chart-h-240"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.journal.h2.mood_dist_chart">Mood distribution (entry count per level)</h2>
            <div id="j-dist-chart" class="chart-h-200"></div>
            <p data-i18n="view.journal.hint.mood_dist" class="muted small">Frequency of each mood level across all entries. Reveals overall sentiment shape — heavy-tailed toward frustrated vs. centered on focused — independent of when each entry happened.</p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.journal.h2.new_entry">New entry</h2>
            ${isGeneral ? '' : `
                <select id="mood" data-tip="view.journal.tip.mood">
                    <option data-i18n="view.journal.opt.no_mood" value="">no mood</option>
                    <option data-i18n="view.journal.opt.2_frustrated" value="-2">-2 frustrated</option>
                    <option data-i18n="view.journal.opt.1_off" value="-1">-1 off</option>
                    <option data-i18n="view.journal.opt.0_neutral" value="0">0 neutral</option>
                    <option data-i18n="view.journal.opt.1_focused" value="1">+1 focused</option>
                    <option data-i18n="view.journal.opt.2_confident" value="2">+2 confident</option>
                </select>
            `}
            <textarea id="body" placeholder="What happened today? Setups taken / missed? Process notes?"
                      data-i18n-placeholder="view.journal.placeholder.body" data-tip="view.journal.tip.body"></textarea>
            <div class="inline-form">
                <button data-i18n="view.journal.btn.save" data-tip="view.journal.tip.save" data-shortcut="journal_save" class="primary" id="save">Save</button>
                <button data-i18n="view.journal.btn.insert_template" data-tip="view.journal.tip.insert_template" class="primary btn-magenta-gradient" id="apply-template">Insert template</button>
            </div>
        </div>
    `;
    renderMoodChart(entries);
    renderMoodDistChart(entries);
    const refreshBtn = mount.querySelector('#journal-refresh-btn');
    if (refreshBtn) refreshBtn.addEventListener('click', () =>
        window.dispatchEvent(new HashChangeEvent('hashchange')));
    const dayInput = mount.querySelector('#journal-day');
    if (dayInput) {
        dayInput.addEventListener('change', (e) => {
            window.location.hash = `journal/${e.target.value}`;
        });
    }
    // Prev / next day shortcuts that mirror Tradervue's day-stepper.
    mount.querySelectorAll('[data-day-step]').forEach(btn => {
        btn.addEventListener('click', () => {
            const step = Number(btn.dataset.dayStep) || 0;
            const d = new Date(day + 'T00:00:00');
            d.setDate(d.getDate() + step);
            const next = d.toISOString().slice(0, 10);
            window.location.hash = `journal/${next}`;
        });
    });
    mount.querySelector('#save').addEventListener('click', async () => {
        const body_md = mount.querySelector('#body').value.trim();
        if (!body_md) {
            showToast(t('view.journal.alert.empty_body'), { level: 'warning' });
            return;
        }
        const mood = mount.querySelector('#mood')?.value;
        try {
            await api.createJournal({
                day: isGeneral ? null : day,
                body_md,
                mood: mood === '' || mood === undefined ? null : Number(mood),
            });
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.journal.toast.saved'), { level: 'success' });
            renderJournalView(mount, state, dayOrGeneral);
        } catch (err) {
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
    mount.querySelector('#apply-template').addEventListener('click', async () => {
        const tpl = await api.defaultNoteTemplate('journal');
        if (!viewIsCurrent(tok)) return;
        const ta = mount.querySelector('#body');
        if (!ta) return;
        if (tpl && tpl.body_md) {
            ta.value = (ta.value ? ta.value + '\n\n' : '') + tpl.body_md;
        } else {
            showToast(t('view.journal.alert.no_template'), { level: 'warning' });
        }
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            if (!await tConfirm('view.journal.confirm.delete', {}, { level: 'danger' })) return;
            try {
                await api.deleteJournal(b.dataset.del);
                if (!viewIsCurrent(tok)) return;
                showToast(t('view.journal.toast.deleted'), { level: 'success' });
                renderJournalView(mount, state, dayOrGeneral);
            } catch (err) {
                showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            }
        }));
    void esc;
}

// ----------------------------------------------------------------------------
// Day summary panel — shown above the entries when the user picks a specific
// day. Mirrors Tradervue's per-day card: net P&L, win/loss split, trade list.
// ----------------------------------------------------------------------------
function dayJourneySummary(day, trades) {
    if (!Array.isArray(trades) || !trades.length) {
        return `
            <div class="journal-day-strip">
                <button type="button" class="link" data-day-step="-1">← ${esc(t('view.journal.day_step.prev'))}</button>
                <span class="muted">${esc(t('view.journal.empty.no_trades', { day }))}</span>
                <button type="button" class="link" data-day-step="1">${esc(t('view.journal.day_step.next'))} →</button>
            </div>
        `;
    }
    let net = 0, gross = 0, fees = 0, vol = 0, wins = 0, losses = 0, scratch = 0;
    let totalHold = 0, holdN = 0;
    for (const tr of trades) {
        const n = Number(tr.net_pnl) || 0;
        net += n;
        gross += Number(tr.gross_pnl) || 0;
        fees  += Number(tr.fees) || 0;
        vol   += Math.abs(Number(tr.qty) || 0) * Math.abs(Number(tr.entry_avg) || 0);
        if (n > 0) wins++;
        else if (n < 0) losses++;
        else scratch++;
        if (tr.opened_at && tr.closed_at) {
            const h = (new Date(tr.closed_at) - new Date(tr.opened_at)) / 1000;
            if (h > 0) { totalHold += h; holdN++; }
        }
    }
    const avgHold = holdN > 0 ? totalHold / holdN : 0;
    return `
        <div class="journal-day-strip">
            <button type="button" class="link" data-day-step="-1">← ${esc(t('view.journal.day_step.prev'))}</button>
            <span class="journal-day-label">${esc(day)}</span>
            <button type="button" class="link" data-day-step="1">${esc(t('view.journal.day_step.next'))} →</button>
        </div>
        <div class="journal-day-summary">
            <div class="jds-card"><div class="jds-label">${esc(t('view.dashboard.stat.net_pnl'))}</div>
                <div class="jds-val ${pnlClass(net)}">${fmtMoney(net)}</div></div>
            <div class="jds-card"><div class="jds-label">${esc(t('view.reports.stat.gross_pnl'))}</div>
                <div class="jds-val ${pnlClass(gross)}">${fmtMoney(gross)}</div></div>
            <div class="jds-card"><div class="jds-label">${esc(t('view.dashboard.stat.fees'))}</div>
                <div class="jds-val">${fmtMoney(fees)}</div></div>
            <div class="jds-card"><div class="jds-label">${esc(t('view.dashboard.stat.trades'))}</div>
                <div class="jds-val">${trades.length}</div></div>
            <div class="jds-card"><div class="jds-label">${esc(t('view.journal.day_summary.wls'))}</div>
                <div class="jds-val">${wins} / ${losses} / ${scratch}</div></div>
            <div class="jds-card"><div class="jds-label">${esc(t('view.dashboard.stat.avg_hold'))}</div>
                <div class="jds-val">${fmtSecs(avgHold)}</div></div>
            <div class="jds-card"><div class="jds-label">${esc(t('view.reports.stat.volume'))}</div>
                <div class="jds-val">${fmtMoney(vol)}</div></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.journal.h2.trades_for_day">Trades for this day</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.search.th.symbol">Symbol</th>
                    <th data-i18n="view.search.th.side">Side</th>
                    <th data-i18n="view.journal.day_summary.opened">Opened</th>
                    <th data-i18n="view.journal.day_summary.closed">Closed</th>
                    <th data-i18n="view.journal.day_summary.qty">Qty</th>
                    <th data-i18n="view.dashboard.stat.net_pnl">Net P&L</th>
                </tr></thead>
                <tbody>${trades.map(tr => `
                    <tr>
                        <td><a href="#trade/${tr.id}">${esc(tr.symbol)}</a></td>
                        <td>${tr.side}</td>
                        <td>${fmtDateTime(tr.opened_at)}</td>
                        <td>${tr.closed_at ? fmtDateTime(tr.closed_at) : '—'}</td>
                        <td>${tr.qty}</td>
                        <td class="${pnlClass(tr.net_pnl)}">${tr.net_pnl !== null && tr.net_pnl !== undefined ? fmtMoney(tr.net_pnl) : '—'}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
    `;
}

function renderMoodChart(entries) {
    const el = document.getElementById('j-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const moods = (entries || [])
        .filter(e => Number.isFinite(Number(e.mood)) && e.created_at)
        .sort((a, b) => new Date(a.created_at) - new Date(b.created_at));
    if (moods.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.journal.empty_chart">${esc(t('view.journal.empty_chart'))}</div>`;
        return;
    }
    const labels = moods.map(e => new Date(e.created_at).toLocaleDateString(undefined, { month: '2-digit', day: '2-digit' }));
    const ys = moods.map(e => Number(e.mood));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: false, range: [-2.5, 2.5] } },
        series: [
            { label: t('view.journal.chart.entry_idx') },
            { label: t('view.journal.chart.mood'),
              stroke: '#b86bff', width: 1.4,
              points: { show: true, size: 10, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.journal.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderMoodDistChart(entries) {
    const el = document.getElementById('j-dist-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const counts = new Map([[-2, 0], [-1, 0], [0, 0], [1, 0], [2, 0]]);
    for (const e of (entries || [])) {
        const m = Number(e.mood);
        if (counts.has(m)) counts.set(m, counts.get(m) + 1);
    }
    const labels = [
        t('view.journal.opt.2_frustrated'),
        t('view.journal.opt.1_off'),
        t('view.journal.opt.0_neutral'),
        t('view.journal.opt.1_focused'),
        t('view.journal.opt.2_confident'),
    ];
    const ys = [-2, -1, 0, 1, 2].map(k => counts.get(k));
    if (!ys.some(v => v > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.journal.empty_dist_chart">${esc(t('view.journal.empty_dist_chart'))}</div>`;
        return;
    }
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.journal.chart.mood_level') },
            { label: t('view.journal.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}
