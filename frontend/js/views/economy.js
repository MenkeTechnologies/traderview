// Economic calendar — upcoming US macro releases grouped by day.
import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderEconomy(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// ECONOMIC CALENDAR</h1>
        <form id="ec-form" class="inline-form">
            <label>Horizon (days)
                <input name="days" type="number" value="60" min="1" max="365" style="width:90px">
            </label>
            <label>Min importance
                <select name="importance">
                    <option value="low">All (low+)</option>
                    <option value="medium" selected>Medium+</option>
                    <option value="high">High only</option>
                </select>
            </label>
            <button class="primary" type="submit">Load</button>
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
        if (ecEl) ecEl.innerHTML = '<div class="boot">loading…</div>';
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
        ecEl.innerHTML = '<p class="muted">No events in horizon.</p>';
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
                <h2>${esc(day)} · ${dayName(day)}</h2>
                <table class="trades">
                    <thead><tr><th>Time (ET)</th><th>Importance</th><th>Event</th>
                        <th>Category</th><th>Country</th></tr></thead>
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
