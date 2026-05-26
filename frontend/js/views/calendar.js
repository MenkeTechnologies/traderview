import { api } from '../api.js';
import { fmtMoney } from '../util.js';

export async function renderCalendar(mount, state) {
    if (!state.accountId) {
        mount.innerHTML = '<p class="boot">No account.</p>';
        return;
    }
    const cells = await api.calendar(state.accountId);
    if (!cells.length) {
        mount.innerHTML = '<p class="boot">No data yet.</p>';
        return;
    }
    const byDay = new Map(cells.map(c => [c.day, c]));
    const max = Math.max(...cells.map(c => Math.abs(Number(c.net_pnl))), 1);

    // Build year-month grid showing all months that have data.
    const months = new Set();
    cells.forEach(c => months.add(c.day.slice(0, 7)));
    const sortedMonths = Array.from(months).sort();

    const monthHtml = sortedMonths.map(ym => {
        const [yyyy, mm] = ym.split('-').map(Number);
        const firstDay = new Date(yyyy, mm - 1, 1);
        const daysInMonth = new Date(yyyy, mm, 0).getDate();
        const startCol = firstDay.getDay(); // 0=Sun
        let cellsRow = '';
        // Padding for first row.
        for (let i = 0; i < startCol; i++) cellsRow += '<div class="cal-cell empty"></div>';
        for (let d = 1; d <= daysInMonth; d++) {
            const date = `${yyyy}-${String(mm).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
            const cell = byDay.get(date);
            if (cell) {
                const v = Number(cell.net_pnl);
                const intensity = Math.min(1, Math.abs(v) / max);
                const color = v >= 0
                    ? `rgba(35, 209, 96, ${0.15 + intensity * 0.7})`
                    : `rgba(255, 56, 96, ${0.15 + intensity * 0.7})`;
                cellsRow += `<div class="cal-cell large" style="background:${color}"
                    title="${date} · ${fmtMoney(v)} · ${cell.trades} trades">${d}</div>`;
            } else {
                cellsRow += `<div class="cal-cell large empty">${d}</div>`;
            }
        }
        return `<div class="cal-month">
            <h3>${monthName(mm)} ${yyyy}</h3>
            <div class="cal-grid">
                <div class="cal-dow">S</div><div class="cal-dow">M</div><div class="cal-dow">T</div>
                <div class="cal-dow">W</div><div class="cal-dow">T</div><div class="cal-dow">F</div>
                <div class="cal-dow">S</div>
                ${cellsRow}
            </div>
        </div>`;
    }).join('');

    mount.innerHTML = `
        <h1 class="view-title">// CALENDAR</h1>
        <div class="cal-months">${monthHtml}</div>
    `;
}

function monthName(m) {
    return ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'][m - 1];
}
