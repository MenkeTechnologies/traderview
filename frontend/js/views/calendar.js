import { api } from '../api.js';
import { esc, fmtMoney } from '../util.js';
import { t } from '../i18n.js';

export async function renderCalendar(mount, state) {
    if (!state.accountId) {
        mount.innerHTML = '<p data-i18n="view.calendar.hint.no_account" class="boot">No account.</p>';
        return;
    }
    const cells = await api.calendar(state.accountId);
    if (!cells.length) {
        mount.innerHTML = '<p data-i18n="view.calendar.hint.no_data_yet" class="boot">No data yet.</p>';
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
                    title="${esc(t('view.dashboard.cal.tooltip', { day: date, pnl: fmtMoney(v), n: cell.trades }))}">${d}</div>`;
            } else {
                cellsRow += `<div class="cal-cell large empty">${d}</div>`;
            }
        }
        return `<div class="cal-month">
            <h3>${esc(t('common.month_year', { month: monthName(mm), year: yyyy }))}</h3>
            <div class="cal-grid">
                <div class="cal-dow">${esc(t('common.dow.short.sun'))}</div><div class="cal-dow">${esc(t('common.dow.short.mon'))}</div><div class="cal-dow">${esc(t('common.dow.short.tue'))}</div>
                <div class="cal-dow">${esc(t('common.dow.short.wed'))}</div><div class="cal-dow">${esc(t('common.dow.short.thu'))}</div><div class="cal-dow">${esc(t('common.dow.short.fri'))}</div>
                <div class="cal-dow">${esc(t('common.dow.short.sat'))}</div>
                ${cellsRow}
            </div>
        </div>`;
    }).join('');

    mount.innerHTML = `
        <h1 data-i18n="view.calendar.h1.calendar" class="view-title">// CALENDAR</h1>
        <div class="cal-months">${monthHtml}</div>
    `;
}

function monthName(m) {
    const keys = ['jan','feb','mar','apr','may','jun','jul','aug','sep','oct','nov','dec'];
    return t(`common.month.${keys[m - 1]}`);
}
