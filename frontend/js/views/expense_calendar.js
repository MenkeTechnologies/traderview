// Tradervue-style monthly EXPENSE calendar — one month at a time, day
// cells show total spend + receipt count, color-tinted by dominant tax
// bucket. Mirrors the layout of `views/calendar.js` (trade journal),
// but tints cells by tax-bucket (business/rental/personal/unclass) so
// the deductibility story is visible at a glance.
//
// Click a day → drill into Purchases pre-filtered to that date.

import { api } from '../api.js';
import { fmtUsd } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import {
    mountBusinessSelector,
    onChange as onBusinessChange,
    activeBusinessId,
} from '../business_context.js';

const DOW_SHORT = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'];
const MONTH_NAMES = ['January','February','March','April','May','June',
                     'July','August','September','October','November','December'];

function esc(s) {
    return String(s).replace(/[&<>"]/g, (c) => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;',
    }[c]));
}

function shortMoney(v) {
    const n = Number(v) || 0;
    if (!n) return '$0';
    if (n >= 1000) return '$' + Math.round(n / 100) / 10 + 'k';
    return '$' + Math.round(n);
}

export async function renderExpenseCalendar(mount) {
    const tok = currentViewToken();
    const now = new Date();
    const stateKey = 'expense_calendar_ym';
    const saved = (localStorage.getItem(stateKey) || '').split('-');
    let year = Number(saved[0]) || now.getFullYear();
    let month = Number(saved[1]) || (now.getMonth() + 1);

    function drawShell() {
        const yearOpts = Array.from({ length: 6 }, (_, i) => now.getFullYear() - i)
            .map((y) => `<option value="${y}"${y === year ? ' selected' : ''}>${y}</option>`).join('');
        const monthOpts = MONTH_NAMES.map((m, i) =>
            `<option value="${i + 1}"${i + 1 === month ? ' selected' : ''}>${m}</option>`).join('');
        mount.innerHTML = `
            <div class="excal-header">
                <h1 class="view-title">
                    <span data-i18n="view.exp_calendar.title">// EXPENSE CALENDAR</span>
                </h1>
                <div class="excal-controls">
                    <span id="excal-biz-selector"></span>
                    <button type="button" id="excal-prev" class="btn btn-secondary btn-compact" aria-label="prev month">‹</button>
                    <select id="excal-month">${monthOpts}</select>
                    <select id="excal-year">${yearOpts}</select>
                    <button type="button" id="excal-next" class="btn btn-secondary btn-compact" aria-label="next month">›</button>
                    <button type="button" id="excal-today" class="btn btn-secondary btn-compact">
                        ${esc(t('view.exp_calendar.today'))}
                    </button>
                </div>
                <div class="excal-legend">
                    <span><span class="excal-swatch excal-biz"></span>${esc(t('view.exp_calendar.legend.business'))}</span>
                    <span><span class="excal-swatch excal-rent"></span>${esc(t('view.exp_calendar.legend.rental'))}</span>
                    <span><span class="excal-swatch excal-pers"></span>${esc(t('view.exp_calendar.legend.personal'))}</span>
                    <span><span class="excal-swatch excal-unc"></span>${esc(t('view.exp_calendar.legend.unclassified'))}</span>
                </div>
            </div>
            <div id="excal-body"><div class="tv-spinner-wrap"><div class="tv-spinner"></div></div></div>
        `;
        mount.querySelector('#excal-prev').addEventListener('click', () => step(-1));
        mount.querySelector('#excal-next').addEventListener('click', () => step(1));
        mount.querySelector('#excal-month').addEventListener('change', (e) => {
            month = Number(e.target.value); save(); load();
        });
        mount.querySelector('#excal-year').addEventListener('change', (e) => {
            year = Number(e.target.value); save(); load();
        });
        mount.querySelector('#excal-today').addEventListener('click', () => {
            year = now.getFullYear(); month = now.getMonth() + 1;
            save(); drawShell(); load();
        });
    }

    function step(delta) {
        month += delta;
        if (month < 1) { month = 12; year -= 1; }
        if (month > 12) { month = 1; year += 1; }
        save();
        drawShell();
        load();
    }
    function save() {
        try { localStorage.setItem(stateKey, `${year}-${month}`); } catch {}
    }

    async function load() {
        let days = [];
        try { days = await api.receiptsMonthCalendar(year, month, activeBusinessId()); }
        catch (e) {
            if (!viewIsCurrent(tok)) return;
            mount.querySelector('#excal-body').innerHTML =
                `<p class="boot">${esc(t('view.exp_calendar.load_failed', { err: e.message }))}</p>`;
            return;
        }
        if (!viewIsCurrent(tok)) return;
        renderMonth(days);
    }

    function renderMonth(days) {
        const firstDay = new Date(year, month - 1, 1);
        const daysInMo = new Date(year, month, 0).getDate();
        const startCol = firstDay.getDay();
        const totalCells = startCol + daysInMo;
        const rows = Math.ceil(totalCells / 7);

        const byDay = new Map();
        let monthlyTotal = 0, monthlyCount = 0, monthlyBiz = 0, monthlyRent = 0, monthlyPers = 0;
        for (const d of days) {
            byDay.set(d.day, d);
            const tot = Number(d.total) || 0;
            monthlyTotal += tot;
            monthlyCount += d.count;
            monthlyBiz += Number(d.business) || 0;
            monthlyRent += Number(d.rental) || 0;
            monthlyPers += Number(d.personal) || 0;
        }

        const dowHead = DOW_SHORT.map(d =>
            `<div class="excal-dow">${esc(t('common.dow.short.' + d, {}, d.toUpperCase()))}</div>`).join('');

        let cellsHtml = '';
        let cursor = 1 - startCol;
        for (let r = 0; r < rows; r++) {
            let weekTotal = 0, weekCount = 0;
            for (let d = 0; d < 7; d++, cursor++) {
                if (cursor < 1 || cursor > daysInMo) {
                    cellsHtml += `<div class="excal-day excal-empty"></div>`;
                    continue;
                }
                const key = `${year}-${String(month).padStart(2,'0')}-${String(cursor).padStart(2,'0')}`;
                const c = byDay.get(key);
                const tot = Number(c?.total) || 0;
                const cnt = c?.count || 0;
                weekTotal += tot; weekCount += cnt;
                const bucket = c?.dominant_bucket || 'none';
                const isToday = (key === todayISO()) ? ' excal-today-cell' : '';
                cellsHtml += `
                    <div class="excal-day excal-bucket-${bucket}${isToday}"
                         data-day="${key}" role="button" tabindex="0"
                         title="${esc(formatTitle(c, key))}">
                        <div class="excal-day-num">${cursor}</div>
                        <div class="excal-day-total">${esc(shortMoney(tot))}</div>
                        <div class="excal-day-count">
                            ${cnt} ${esc(t(cnt === 1 ? 'view.exp_calendar.receipt' : 'view.exp_calendar.receipts'))}
                        </div>
                        ${miniSplitBar(c)}
                    </div>`;
            }
            cellsHtml += `
                <div class="excal-week-total">
                    <div class="excal-week-label">${esc(t('view.exp_calendar.week', { n: r + 1 }))}</div>
                    <div class="excal-day-total">${esc(shortMoney(weekTotal))}</div>
                    <div class="excal-day-count">${weekCount} ${esc(t('view.exp_calendar.receipts'))}</div>
                </div>`;
        }

        const bizPct = monthlyTotal > 0 ? (monthlyBiz / monthlyTotal * 100).toFixed(1) : '0';
        const rentPct = monthlyTotal > 0 ? (monthlyRent / monthlyTotal * 100).toFixed(1) : '0';
        const persPct = monthlyTotal > 0 ? (monthlyPers / monthlyTotal * 100).toFixed(1) : '0';

        mount.querySelector('#excal-body').innerHTML = `
            <div class="excal-monthly-strip">
                <div class="excal-monthly-cell">
                    <div class="excal-monthly-label">${esc(t('view.exp_calendar.month_total'))}</div>
                    <div class="excal-monthly-value">${esc(fmtUsd(monthlyTotal))}</div>
                    <div class="muted small">${monthlyCount} ${esc(t('view.exp_calendar.receipts'))}</div>
                </div>
                <div class="excal-monthly-cell">
                    <div class="excal-monthly-label">${esc(t('view.exp_calendar.business'))}</div>
                    <div class="excal-monthly-value tw-refund">${esc(fmtUsd(monthlyBiz))}</div>
                    <div class="muted small">${bizPct}%</div>
                </div>
                <div class="excal-monthly-cell">
                    <div class="excal-monthly-label">${esc(t('view.exp_calendar.rental'))}</div>
                    <div class="excal-monthly-value">${esc(fmtUsd(monthlyRent))}</div>
                    <div class="muted small">${rentPct}%</div>
                </div>
                <div class="excal-monthly-cell">
                    <div class="excal-monthly-label">${esc(t('view.exp_calendar.personal'))}</div>
                    <div class="excal-monthly-value">${esc(fmtUsd(monthlyPers))}</div>
                    <div class="muted small">${persPct}%</div>
                </div>
            </div>
            <div class="excal-grid">
                ${dowHead}
                <div class="excal-dow">${esc(t('view.exp_calendar.total'))}</div>
                ${cellsHtml}
            </div>
        `;
        // Click → drill into Purchases for that day.
        mount.querySelectorAll('.excal-day[data-day]').forEach((cell) => {
            const go = () => {
                const day = cell.dataset.day;
                window.location.hash = `purchases?from=${day}&to=${day}`;
                try { showToast(t('view.exp_calendar.opened_day', { day }), { level: 'info' }); } catch {}
            };
            cell.addEventListener('click', go);
            cell.addEventListener('keydown', (e) => {
                if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); go(); }
            });
        });
    }

    function formatTitle(c, key) {
        if (!c) return key + ' — no receipts';
        const parts = [key, `total ${shortMoney(c.total)}`, `${c.count} rcpt`];
        if (Number(c.business) > 0) parts.push(`biz ${shortMoney(c.business)}`);
        if (Number(c.rental) > 0) parts.push(`rent ${shortMoney(c.rental)}`);
        if (Number(c.personal) > 0) parts.push(`pers ${shortMoney(c.personal)}`);
        if (Number(c.unclassified) > 0) parts.push(`uncl ${shortMoney(c.unclassified)}`);
        return parts.join(' · ');
    }

    function miniSplitBar(c) {
        if (!c || !Number(c.total)) return '';
        const tot = Number(c.total) || 1;
        const b = (Number(c.business) || 0) / tot * 100;
        const r = (Number(c.rental) || 0) / tot * 100;
        const p = (Number(c.personal) || 0) / tot * 100;
        const u = (Number(c.unclassified) || 0) / tot * 100;
        return `<div class="excal-mini-bar">
            ${b > 0 ? `<span class="excal-bar-biz" style="width:${b}%"></span>` : ''}
            ${r > 0 ? `<span class="excal-bar-rent" style="width:${r}%"></span>` : ''}
            ${p > 0 ? `<span class="excal-bar-pers" style="width:${p}%"></span>` : ''}
            ${u > 0 ? `<span class="excal-bar-unc" style="width:${u}%"></span>` : ''}
        </div>`;
    }

    function todayISO() {
        const d = new Date();
        return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
    }

    drawShell();
    // Mount business selector + re-load month when user switches.
    const bizHost = mount.querySelector('#excal-biz-selector');
    if (bizHost) mountBusinessSelector(bizHost);
    const unsubBiz = onBusinessChange(() => load());
    mount.__excalUnsubBiz = unsubBiz;
    await load();
}
