import { api } from '../api.js';
import { esc, fmtMoney, pnlClass } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const ACTIVE_MONTH_KEY = 'calendar_active_month';
const ACTIVE_YEAR_KEY  = 'calendar_active_year';
const DOW_SHORT = ['sun','mon','tue','wed','thu','fri','sat'];

function monthName(m) {
    const keys = ['jan','feb','mar','apr','may','jun','jul','aug','sep','oct','nov','dec'];
    return t(`common.month.${keys[m - 1]}`);
}

function getActiveYearMonth(cells) {
    const storedY = Number(sessionStorage.getItem(ACTIVE_YEAR_KEY));
    const storedM = Number(sessionStorage.getItem(ACTIVE_MONTH_KEY));
    if (storedY > 0 && storedM >= 1 && storedM <= 12) return { y: storedY, m: storedM };
    // Default: most recent month with trades, fallback to current.
    const withData = cells.filter(c => Number(c.trades) > 0).sort((a, b) => b.day.localeCompare(a.day));
    if (withData[0]) {
        const [y, m] = withData[0].day.split('-').map(Number);
        return { y, m };
    }
    const now = new Date();
    return { y: now.getFullYear(), m: now.getMonth() + 1 };
}

function setActive(y, m) {
    sessionStorage.setItem(ACTIVE_YEAR_KEY,  String(y));
    sessionStorage.setItem(ACTIVE_MONTH_KEY, String(m));
}

export async function renderCalendar(mount, state) {
    const tok = currentViewToken();
    if (!state.accountId) {
        mount.innerHTML = '<p data-i18n="view.calendar.hint.no_account" class="boot">No account.</p>';
        return;
    }
    const cells = await api.calendar(state.accountId);
    if (!viewIsCurrent(tok)) return;
    if (!cells.length) {
        mount.innerHTML = '<p data-i18n="view.calendar.hint.no_data_yet" class="boot">No data yet.</p>';
        return;
    }

    const byDay = new Map(cells.map(c => [c.day, c]));
    const yearsSet = new Set(cells.map(c => Number(c.day.slice(0, 4))));
    const years = [...yearsSet].sort();
    const { y: activeY, m: activeM } = getActiveYearMonth(cells);
    const renderYear = years.includes(activeY) ? activeY : years[years.length - 1];

    mount.innerHTML = `
        <div class="cal-tv-header">
            <h1 class="view-title" style="margin:0"><span data-i18n="view.calendar.h1.calendar">// CALENDAR</span></h1>
            <div class="cal-tv-year" role="tablist">
                ${years.map(y => `<button type="button" data-year="${y}" class="${y === renderYear ? 'active' : ''}">${y}</button>`).join('')}
            </div>
        </div>
        ${activeMonthHtml(byDay, activeY, activeM)}
        <div class="cal-tv-thumbs">
            ${[...Array(12)].map((_, i) => thumbnailHtml(byDay, renderYear, i + 1, activeY === renderYear && activeM === (i + 1))).join('')}
        </div>
    `;

    mount.querySelectorAll('.cal-tv-year button[data-year]').forEach(btn => {
        btn.addEventListener('click', () => {
            const y = Number(btn.dataset.year);
            setActive(y, 1);
            renderCalendar(mount, state);
        });
    });
    mount.querySelectorAll('.cal-tv-thumb-open[data-y][data-m]').forEach(btn => {
        btn.addEventListener('click', () => {
            setActive(Number(btn.dataset.y), Number(btn.dataset.m));
            renderCalendar(mount, state);
            window.scrollTo({ top: 0, behavior: 'smooth' });
        });
    });
}

function activeMonthHtml(byDay, year, month) {
    const firstDay   = new Date(year, month - 1, 1);
    const daysInMo   = new Date(year, month, 0).getDate();
    const startCol   = firstDay.getDay();
    const totalCells = startCol + daysInMo;
    const rows       = Math.ceil(totalCells / 7);

    let monthlyPnl = 0, monthlyTrades = 0;
    for (let d = 1; d <= daysInMo; d++) {
        const key = `${year}-${String(month).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
        const c = byDay.get(key);
        if (c) { monthlyPnl += Number(c.net_pnl) || 0; monthlyTrades += Number(c.trades) || 0; }
    }

    let cellsHtml = '';
    let cursor = 1 - startCol;
    for (let r = 0; r < rows; r++) {
        let weekPnl = 0, weekTrades = 0;
        for (let d = 0; d < 7; d++, cursor++) {
            if (cursor < 1 || cursor > daysInMo) {
                cellsHtml += `<div class="cal-tv-day"><div class="cal-tv-day-num muted"></div></div>`;
                continue;
            }
            const key = `${year}-${String(month).padStart(2, '0')}-${String(cursor).padStart(2, '0')}`;
            const c = byDay.get(key);
            const v = Number(c?.net_pnl) || 0;
            const tr = Number(c?.trades) || 0;
            weekPnl += v; weekTrades += tr;
            const cls = tr === 0 ? 'zero' : v > 0 ? 'pos' : v < 0 ? 'neg' : '';
            cellsHtml += `
                <div class="cal-tv-day ${cls}">
                    <div class="cal-tv-day-num">${cursor}</div>
                    <div class="cal-tv-day-pnl ${pnlClass(v)}">${tr === 0 ? '$0' : fmtMoney(v)}</div>
                    <div class="cal-tv-day-trades">${tr} ${tr === 1 ? 'trade' : 'trades'}</div>
                </div>`;
        }
        cellsHtml += `
            <div class="cal-tv-week-total">
                <div class="cal-tv-week-label">${esc(t('view.calendar.tv.week', { n: r + 1 }))}</div>
                <div class="cal-tv-day-pnl ${pnlClass(weekPnl)}" style="margin-top:auto">${weekTrades === 0 ? '$0' : fmtMoney(weekPnl)}</div>
                <div class="cal-tv-day-trades">${weekTrades} ${weekTrades === 1 ? 'trade' : 'trades'}</div>
            </div>`;
    }

    return `
        <div class="cal-tv-active">
            <div class="cal-tv-active-head">
                <div class="cal-tv-active-month">${esc(monthName(month))}, ${year}</div>
                <div class="cal-tv-active-pnl">${esc(t('view.calendar.tv.monthly_pnl'))}: <span class="${pnlClass(monthlyPnl)}">${fmtMoney(monthlyPnl)}</span></div>
            </div>
            <div class="cal-tv-active-grid">
                ${DOW_SHORT.map(d => `<div class="cal-tv-active-dow">${esc(t('common.dow.short.' + d))}</div>`).join('')}
                <div class="cal-tv-active-dow">${esc(t('view.calendar.tv.total'))}</div>
                ${cellsHtml}
            </div>
        </div>
    `;
}

function thumbnailHtml(byDay, year, month, isActive) {
    const firstDay = new Date(year, month - 1, 1);
    const daysInMo = new Date(year, month, 0).getDate();
    const startCol = firstDay.getDay();
    const totalCells = startCol + daysInMo;
    const rows = Math.ceil(totalCells / 7);

    let grid = '';
    for (let i = 0; i < rows * 7; i++) {
        const day = i - startCol + 1;
        if (day < 1 || day > daysInMo) {
            grid += `<div class="cal-tv-thumb-cell muted"></div>`;
            continue;
        }
        const key = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
        const c = byDay.get(key);
        const v = Number(c?.net_pnl) || 0;
        const tr = Number(c?.trades) || 0;
        const cls = tr === 0 ? 'muted' : v > 0 ? 'pos' : v < 0 ? 'neg' : 'muted';
        grid += `<div class="cal-tv-thumb-cell ${cls}">${day}</div>`;
    }
    return `
        <div class="cal-tv-thumb">
            <div class="cal-tv-thumb-head">
                <div class="cal-tv-thumb-name">${esc(monthName(month))}, ${year}</div>
                <button type="button" class="cal-tv-thumb-open" data-y="${year}" data-m="${month}">
                    ${isActive ? esc(t('view.calendar.tv.active')) : esc(t('view.calendar.tv.open'))}
                </button>
            </div>
            <div class="cal-tv-thumb-grid">
                ${DOW_SHORT.map(d => `<div class="cal-tv-thumb-dow">${esc(t('common.dow.short.' + d))}</div>`).join('')}
                ${grid}
            </div>
        </div>
    `;
}
