// Economic calendar — upcoming US macro releases grouped by day.
import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

export async function renderEconomy(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.economy.h1.economic_calendar" class="view-title">// ECONOMIC CALENDAR</h1>
        <form id="ec-form" class="inline-form">
            <label><span data-i18n="view.economy.label.horizon">Horizon (days)</span>
                <input name="days" type="number" value="60" min="1" max="365" style="width:90px">
            </label>
            <label><span data-i18n="view.economy.label.min_importance">Min importance</span>
                <select name="importance">
                    <option data-i18n="view.economy.opt.all_low" value="low">All (low+)</option>
                    <option data-i18n="view.economy.opt.medium" value="medium" selected>Medium+</option>
                    <option data-i18n="view.economy.opt.high_only" value="high">High only</option>
                </select>
            </label>
            <button data-i18n="view.economy.btn.load" class="primary" type="submit">Load</button>
        </form>
        <div id="ec">loading…</div>
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
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const ecEl2 = mount.querySelector('#ec');
            if (ecEl2) ecEl2.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
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
    const html = Array.from(groups.entries())
        .sort((a, b) => a[0].localeCompare(b[0]))
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
    ecEl.innerHTML = html;
}

function dayName(iso) {
    const d = new Date(iso + 'T00:00:00');
    return d.toLocaleDateString(undefined, { weekday: 'long' });
}
