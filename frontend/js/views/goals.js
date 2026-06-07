// Trading goals dashboard — set targets, see progress + run-rate projection.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t, applyUiI18n } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';

const PACE_COLOR = {
    on_track:      '#7af0a8',
    exceeded:      '#00ffaa',
    falling_short: '#ff1f7a',
    no_target:     '#444',
};

export async function renderGoals(mount, state) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.goals.h1.goals" class="view-title">// GOALS</h1>
        <p class="muted small" data-i18n="view.goals.hint.intro">Set monthly / quarterly / yearly P/L + win-rate + max-drawdown targets. Progress is computed live from closed trades whose opened_at falls in the window. Run-rate projection extrapolates current P/L pace to the end of the period so you know if you're on track today.</p>

        <div class="chart-panel">
            <h2 data-i18n="view.goals.h2.create_goal">Create goal</h2>
            <form id="g-form" class="inline-form">
                <input name="name" placeholder="name" data-i18n-placeholder="common.placeholder.name"
                       data-tip="view.goals.tip.name" data-shortcut="goals_focus_name" required style="min-width:180px;">
                <select name="period" data-tip="view.goals.tip.period">
                    <option data-i18n="view.goals.opt.monthly" value="monthly">monthly</option>
                    <option data-i18n="view.goals.opt.quarterly" value="quarterly">quarterly</option>
                    <option data-i18n="view.goals.opt.yearly" value="yearly">yearly</option>
                    <option data-i18n="view.goals.opt.custom" value="custom">custom</option>
                </select>
                <label><span data-i18n="view.goals.label.account">Account</span>
                    <select name="account_id" data-tip="view.goals.tip.account">
                        <option data-i18n="view.goals.opt.all" value="">(all)</option>
                        ${state.accounts.map(a => `<option value="${a.id}">${esc(a.broker)} · ${esc(a.name)}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.goals.label.start">Start</span>
                    <input name="start_date" type="date" required style="width:140px;" data-tip="view.goals.tip.start"></label>
                <label><span data-i18n="view.goals.label.end">End</span>
                    <input name="end_date"   type="date" required style="width:140px;" data-tip="view.goals.tip.end"></label>
                <label><span data-i18n="view.goals.label.target_pnl">Target P/L $</span>
                    <input name="target_pnl" type="number" step="0.01" style="width:110px;" data-tip="view.goals.tip.target_pnl"></label>
                <label><span data-i18n="view.goals.label.target_win">Target win %</span>
                    <input name="target_win_rate" type="number" min="0" max="100" step="0.5" style="width:90px;" placeholder="60" data-tip="view.goals.tip.target_win"></label>
                <label><span data-i18n="view.goals.label.max_dd">Max DD %</span>
                    <input name="target_max_drawdown_pct" type="number" min="0" max="100" step="0.5" style="width:90px;" placeholder="10" data-tip="view.goals.tip.max_dd"></label>
                <button data-i18n="view.goals.btn.create" data-tip="view.goals.tip.create" class="primary" type="submit">Create</button>
                <span id="g-status" class="muted small"></span>
            </form>
        </div>

        <div id="g-list"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
        <div class="chart-panel">
            <h2 data-i18n="view.goals.h2.progress_chart">P/L progress vs window elapsed per goal</h2>
            <div id="g-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.goals.h2.pace_chart">P/L pace distribution across all goals</h2>
            <div id="g-pace-chart" style="width:100%;height:220px"></div>
        </div>
    `;
    // Default date range based on selected period.
    const periodSel = mount.querySelector('#g-form [name=period]');
    const start = mount.querySelector('#g-form [name=start_date]');
    const end   = mount.querySelector('#g-form [name=end_date]');
    const fillRange = () => {
        const now = new Date();
        let s, e;
        if (periodSel.value === 'monthly') {
            s = new Date(now.getFullYear(), now.getMonth(), 1);
            e = new Date(now.getFullYear(), now.getMonth() + 1, 0);
        } else if (periodSel.value === 'quarterly') {
            const q = Math.floor(now.getMonth() / 3) * 3;
            s = new Date(now.getFullYear(), q, 1);
            e = new Date(now.getFullYear(), q + 3, 0);
        } else if (periodSel.value === 'yearly') {
            s = new Date(now.getFullYear(), 0, 1);
            e = new Date(now.getFullYear(), 11, 31);
        } else { return; }
        start.value = s.toISOString().slice(0, 10);
        end.value   = e.toISOString().slice(0, 10);
    };
    periodSel.addEventListener('change', fillRange);
    fillRange();

    mount.querySelector('#g-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const winPct = fd.get('target_win_rate');
        const body = {
            account_id: fd.get('account_id') || null,
            name: fd.get('name').trim(),
            period: fd.get('period'),
            start_date: fd.get('start_date'),
            end_date: fd.get('end_date'),
            target_pnl: fd.get('target_pnl') ? Number(fd.get('target_pnl')) : null,
            target_win_rate: winPct ? Number(winPct) / 100 : null,
            target_max_drawdown_pct: fd.get('target_max_drawdown_pct') ? Number(fd.get('target_max_drawdown_pct')) : null,
        };
        const status = mount.querySelector('#g-status');
        if (status) status.textContent = t('common.status.saving');
        try {
            await api.createGoal(body);
            if (!viewIsCurrent(tok)) return;
            e.target.reset();
            fillRange();
            await refresh(mount, tok);
            if (!viewIsCurrent(tok)) return;
            const s2 = mount.querySelector('#g-status');
            if (s2) s2.textContent = '';
            showToast(t('view.goals.toast.created', { name: body.name }), { level: 'success' });
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const s2 = mount.querySelector('#g-status');
            if (s2) s2.textContent = t('common.error', { err: err.message });
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    const el = mount.querySelector('#g-list');
    if (!el) return;
    try {
        const goals = await api.listGoals();
        if (!viewIsCurrent(tok)) return;
        if (!goals.length) { el.innerHTML = '<div class="chart-panel"><p data-i18n="view.goals.hint.no_goals_yet_create_one_above" class="muted small">No goals yet — create one above.</p></div>'; return; }
        // Fetch progress for each goal in parallel.
        const progressList = await Promise.all(goals.map(g =>
            api.goalProgress(g.id).catch(() => null)));
        if (!viewIsCurrent(tok)) return;
        const el2 = mount.querySelector('#g-list');
        if (!el2) return;
        el2.innerHTML = progressList.map((p, i) => p ? card(p) : `<div class="chart-panel">
            <p class="boot">${esc(t('view.goals.boot.progress_failed', { name: goals[i].name }))}</p></div>`).join('');
        try { applyUiI18n(el2); } catch (_) {}
        renderProgressChart(progressList.filter(Boolean));
        renderPaceChart(progressList.filter(Boolean));
        el2.querySelectorAll('.g-del').forEach(b => {
            b.addEventListener('click', async () => {
                if (!await tConfirm('view.goals.confirm.delete', {}, { level: 'danger' })) return;
                try {
                    await api.deleteGoal(b.dataset.id);
                    showToast(t('view.goals.toast.deleted'), { level: 'success' });
                    if (viewIsCurrent(tok)) await refresh(mount, tok);
                } catch (e) { showToast(t('common.error', { err: e.message }), { level: 'error' }); }
            });
        });
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el2 = mount.querySelector('#g-list');
        if (el2) el2.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderProgressChart(progresses) {
    const el = document.getElementById('g-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const items = (progresses || []).filter(p => Number.isFinite(Number(p.elapsed_pct)));
    if (items.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.goals.empty_chart">${esc(t('view.goals.empty_chart'))}</div>`;
        return;
    }
    const labels = items.map(p => p.goal.name);
    const elapsed = items.map(p => Number(p.elapsed_pct));
    const pnlDone = items.map(p => Number.isFinite(Number(p.pnl_pct_complete)) ? Number(p.pnl_pct_complete) : null);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: false, range: [0, 120] } },
        series: [
            { label: t('view.goals.chart.goal_idx') },
            { label: t('view.goals.chart.elapsed'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 12, fill: '#ffd84a', stroke: '#ffd84a' } },
            { label: t('view.goals.chart.pnl_done'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, elapsed, pnlDone], el);
}

function renderPaceChart(progresses) {
    const el = document.getElementById('g-pace-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!progresses || progresses.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.goals.empty_pace_chart">${esc(t('view.goals.empty_pace_chart'))}</div>`;
        return;
    }
    const counts = { on_track: 0, exceeded: 0, falling_short: 0, no_target: 0 };
    for (const p of progresses) {
        const k = p.pnl_pace;
        if (k in counts) counts[k] += 1;
    }
    const labels = [
        t('view.goals.chart.pace.on_track'),
        t('view.goals.chart.pace.exceeded'),
        t('view.goals.chart.pace.falling_short'),
        t('view.goals.chart.pace.no_target'),
    ];
    const ys = [counts.on_track, counts.exceeded, counts.falling_short, counts.no_target];
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.goals.chart.pace_idx') },
            { label: t('view.goals.chart.goal_count'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function paceChip(pace) {
    const key = `view.goals.pace.${pace}`;
    const v = t(key);
    const label = v && v !== key ? v : pace;
    return `<span style="background:${PACE_COLOR[pace] || '#444'};color:#000;padding:1px 6px;border-radius:2px;font-size:10px;">
        ${esc(label)}
    </span>`;
}

function progressBar(actual, target, isLowerBetter) {
    if (target == null) return `<div class="muted small">${esc(t('view.goals.empty.no_target'))}</div>`;
    const pct = Math.max(0, Math.min(100, (Math.abs(actual) / Math.abs(target)) * 100));
    const meets = isLowerBetter ? actual <= target : actual >= target;
    const color = meets ? '#7af0a8' : pct >= 80 ? '#ffd24a' : '#ff7a1f';
    return `<div style="height:14px;background:#1a1d2e;border:1px solid var(--border);position:relative;">
        <div style="width:${pct}%;height:100%;background:${color};"></div>
    </div>`;
}

function card(p) {
    const g = p.goal;
    const targetPnl = g.target_pnl != null ? Number(g.target_pnl) : null;
    const targetWin = g.target_win_rate ? g.target_win_rate * 100 : null;
    const targetDD  = g.target_max_drawdown_pct ?? null;
    const acct = g.account_id ? '' : t('view.goals.h2.all_accounts_suffix');
    return `<div class="chart-panel" style="margin-bottom:10px;">
        <div style="display:flex;align-items:center;justify-content:space-between;">
            <h2 style="margin:0;">${esc(g.name)} <span class="muted small">${esc(g.period)} · ${g.start_date} → ${g.end_date}${acct}</span></h2>
            <button data-i18n="view.goals.btn.delete" class="btn g-del" data-id="${g.id}">Delete</button>
        </div>
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.goals.card.window_progress">Window progress</div>
                <div class="value">${p.elapsed_pct.toFixed(0)}%</div>
                <div class="small muted">${esc(t('view.goals.card.days_progress', { elapsed: p.days_elapsed, total: p.days_total }))}</div></div>
            <div class="card"><div class="label" data-i18n="view.goals.card.closed_trades">Closed trades</div>
                <div class="value">${p.trades_in_window}</div>
                <div class="small muted">${p.wins}W / ${p.losses}L</div></div>
            <div class="card"><div class="label" data-i18n="view.goals.card.actual_pnl">Actual P/L</div>
                <div class="value ${p.actual_pnl >= 0 ? 'pos' : 'neg'}">$${fmt(p.actual_pnl)}</div>
                ${p.projected_pnl != null ? `<div class="small muted">${esc(t('view.goals.card.projected_end', { amount: fmt(p.projected_pnl) }))}</div>` : ''}
            </div>
            <div class="card"><div class="label" data-i18n="view.goals.card.win_rate">Win rate</div>
                <div class="value">${(p.actual_win_rate * 100).toFixed(1)}%</div></div>
            <div class="card"><div class="label" data-i18n="view.goals.card.max_drawdown">Max drawdown</div>
                <div class="value ${p.actual_max_drawdown_pct > (targetDD ?? Infinity) ? 'neg' : ''}">
                    ${p.actual_max_drawdown_pct.toFixed(2)}%
                </div></div>
        </div>

        <div style="display:grid;grid-template-columns:160px 1fr 160px;gap:6px;font-size:11px;align-items:center;margin-top:8px;">
            <div>P/L vs target ${targetPnl != null ? '$' + fmt(targetPnl) : ''}</div>
            <div>${progressBar(p.actual_pnl, targetPnl, false)}</div>
            <div>${paceChip(p.pnl_pace)}${p.pnl_pct_complete != null ? ` <span class="muted">${esc(t('view.goals.card.pct_done', { pct: p.pnl_pct_complete.toFixed(1) }))}</span>` : ''}</div>

            <div>${esc(t('view.goals.card.win_rate_vs', { target: targetWin != null ? targetWin.toFixed(1) + '%' : '—' }))}</div>
            <div>${targetWin != null ? progressBar(p.actual_win_rate * 100, targetWin, false) : `<div class="muted small">${esc(t('view.goals.card.no_target'))}</div>`}</div>
            <div>${paceChip(p.win_rate_pace)}</div>

            <div>${esc(t('view.goals.card.max_dd_vs_cap', { target: targetDD != null ? targetDD.toFixed(1) + '%' : '—' }))}</div>
            <div>${targetDD != null ? progressBar(p.actual_max_drawdown_pct, targetDD, true) : `<div class="muted small">${esc(t('view.goals.card.no_cap'))}</div>`}</div>
            <div>${paceChip(p.drawdown_pace)}</div>
        </div>
        ${g.notes ? `<p class="muted small" style="margin-top:6px;">${esc(g.notes)}</p>` : ''}
    </div>`;
}
