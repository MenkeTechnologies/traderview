// Economic calendar — upcoming US macro releases grouped by day.
import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';

export async function renderEconomy(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.economy.h1.economic_calendar" class="view-title">// ECONOMIC CALENDAR</h1>
        <form id="ec-form" class="inline-form">
            <label><span data-i18n="view.economy.label.horizon">Horizon (days)</span>
                <input name="days" type="number" value="60" min="1" max="365" style="width:90px" data-tip="view.economy.tip.horizon">
            </label>
            <label><span data-i18n="view.economy.label.min_importance">Min importance</span>
                <select name="importance" data-tip="view.economy.tip.importance">
                    <option data-i18n="view.economy.opt.all_low" value="low">All (low+)</option>
                    <option data-i18n="view.economy.opt.medium" value="medium" selected>Medium+</option>
                    <option data-i18n="view.economy.opt.high_only" value="high">High only</option>
                </select>
            </label>
            <button data-i18n="view.economy.btn.load" data-tip="view.economy.tip.load" data-shortcut="economy_load" class="primary" type="submit">Load</button>
        </form>
        <div id="ec"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
    `;
    const load = async () => {
        const form = mount.querySelector('#ec-form');
        if (!form) return;
        const fd = new FormData(form);
        const days = Number(fd.get('days') || 60);
        const imp = fd.get('importance');
        const ecEl = mount.querySelector('#ec');
        if (ecEl) ecEl.innerHTML = '<div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.status.loading">loading…</div></div>';
        try {
            const evs = await api.economyCalendar(days, imp);
            if (!viewIsCurrent(tok)) return;
            renderEvents(evs, mount);
            showToast(t('view.economy.toast.loaded', { count: evs.length, days }), { level: evs.length > 0 ? 'success' : 'info' });
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const ecEl2 = mount.querySelector('#ec');
            if (ecEl2) ecEl2.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
            showToast(t('toast.error.api', { err: e.message }), { level: 'error' });
        }
    };
    mount.querySelector('#ec-form').addEventListener('submit', (e) => { e.preventDefault(); load(); });
    load();
}

function renderEvents(evs, mount) {
    const ecEl = mount.querySelector('#ec');
    if (!ecEl) return;
    if (!evs.length) {
        ecEl.innerHTML = '<p data-i18n="view.economy.hint.no_events_in_horizon" class="muted">No events in horizon.</p>';
        return;
    }
    // Group by day.
    const groups = new Map();
    for (const e of evs) {
        const day = e.when_et.slice(0, 10);
        const arr = groups.get(day) || [];
        arr.push(e);
        groups.set(day, arr);
    }
    const sortedDays = Array.from(groups.entries()).sort((a, b) => a[0].localeCompare(b[0]));
    const chartHtml = `
        <div class="chart-panel">
            <h2 data-i18n="view.economy.h2.density">Events per day (high-importance overlay)</h2>
            <div id="ec-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.economy.h2.category_chart">Event count by category — which macro themes dominate the horizon</h2>
            <div id="ec-category-chart" style="width:100%;height:220px"></div>
        </div>`;
    const html = sortedDays
        .map(([day, items]) => `
            <div class="chart-panel">
                <h2>${esc(t('view.economy.h2.day', { day, dayName: dayName(day) }))}</h2>
                <table class="trades">
                    <thead><tr><th data-i18n="view.economy.th.time_et">Time (ET)</th><th data-i18n="view.economy.th.importance">Importance</th><th data-i18n="view.economy.th.event">Event</th>
                        <th data-i18n="view.economy.th.category">Category</th><th data-i18n="view.economy.th.country">Country</th></tr></thead>
                    <tbody>${items.map(e => `
                        <tr>
                            <td>${esc(e.when_et.slice(11, 16))}</td>
                            <td><span class="econ-imp ${e.importance}">${e.importance}</span></td>
                            <td><strong>${esc(e.name)}</strong></td>
                            <td>${esc(e.category)}</td>
                            <td>${esc(e.country)}</td>
                        </tr>`).join('')}</tbody>
                </table>
            </div>`).join('');
    ecEl.innerHTML = chartHtml + html;
    renderDensityChart(sortedDays);
    renderCategoryChart(evs);
}

function renderCategoryChart(evs) {
    const el = document.getElementById('ec-category-chart');
    if (!el || !window.uPlot || !Array.isArray(evs) || evs.length < 1) {
        if (el) el.innerHTML = `<div class="muted" data-i18n="view.economy.empty_category_chart">${esc(t('view.economy.empty_category_chart'))}</div>`;
        return;
    }
    el.innerHTML = '';
    const counts = new Map();
    for (const e of evs) {
        const c = e.category || '?';
        counts.set(c, (counts.get(c) || 0) + 1);
    }
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]).slice(0, 20);
    const labels = pairs.map(([k]) => k);
    const ys = pairs.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.economy.chart.category_idx') },
            { label: t('view.economy.chart.event_count'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderDensityChart(sortedDays) {
    const el = document.getElementById('ec-chart');
    if (!el || !window.uPlot || !sortedDays || sortedDays.length < 1) {
        if (el) el.innerHTML = `<div class="muted" data-i18n="view.economy.empty_chart">${esc(t('view.economy.empty_chart'))}</div>`;
        return;
    }
    el.innerHTML = '';
    const labels = sortedDays.map(([d]) => d.slice(5));
    const total = sortedDays.map(([, items]) => items.length);
    const high = sortedDays.map(([, items]) => items.filter(e => e.importance === 'high').length);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.economy.chart.day_idx') },
            { label: t('view.economy.chart.total'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.economy.chart.high'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 8, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, total, high], el);
}

function dayName(iso) {
    const d = new Date(iso + 'T00:00:00');
    return d.toLocaleDateString(undefined, { weekday: 'long' });
}
