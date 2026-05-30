// Earnings calendar + surprise leaderboard.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const DOW_SLUGS = ['sun','mon','tue','wed','thu','fri','sat'];
const dowLabel = (i) => t('common.dow.' + DOW_SLUGS[i]);

export async function renderEarningsCal(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.earnings_cal.h1.earnings_calendar" class="view-title">// EARNINGS CALENDAR</h1>
        <p class="muted small" data-i18n="view.earnings_cal.hint.intro">Polls Yahoo's quoteSummary earnings module every 6h across your watchlist symbols. Surprise % = (actual − estimate) / |estimate| × 100. Reaction columns are next-session and 5-session price moves vs the close on / just before the earnings date, computed from cached daily bars.</p>

        <div class="chart-panel">
            <form class="inline-form" id="e-form">
                <label><span data-i18n="view.earnings_cal.label.upcoming_days">Upcoming days</span>
                    <input name="days" type="number" min="1" max="30" value="7" style="width:80px;">
                </label>
                <label><span data-i18n="view.earnings_cal.label.surprise_lookback">Surprise lookback (days)</span>
                    <input name="back" type="number" min="1" max="365" value="30" style="width:90px;">
                </label>
                <button data-i18n="view.earnings_cal.btn.refresh_view" class="primary" type="submit">Refresh view</button>
                <button data-i18n="view.earnings_cal.btn.poll_now_yahoo" class="btn" type="button" id="e-poll">Poll now (Yahoo)</button>
                <span id="e-status" class="muted small"></span>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.earnings_cal.h2.this_week_calendar_matrix">This week (calendar matrix)</h2>
            <div id="e-cal"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.earnings_cal.h2.biggest_surprises_recent">Biggest surprises (recent)</h2>
            <div id="e-surp"></div>
        </div>
    `;
    mount.querySelector('#e-form').addEventListener('submit', (e) => {
        e.preventDefault();
        refresh(mount, tok);
    });
    mount.querySelector('#e-poll').addEventListener('click', async () => {
        const status = mount.querySelector('#e-status');
        if (status) status.textContent = t('common.status.polling');
        try {
            const s = await api.earningsPollNow();
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#e-status');
            if (status2) status2.textContent = t('view.earnings_cal.status.result', { symbols: s.symbols_polled, events: s.events_upserted, reactions: s.reactions_computed });
            await refresh(mount, tok);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const status2 = mount.querySelector('#e-status');
            if (status2) status2.textContent = t('common.error', { err: err.message });
        }
    });
    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    const form = mount.querySelector('#e-form');
    if (!form) return;
    const days = Number(form.days.value) || 7;
    const back = Number(form.back.value) || 30;
    try {
        const [upcoming, surprises] = await Promise.all([
            api.earningsCalendar(days),
            api.earningsSurprises(back),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderCalendarMatrix(upcoming, days, mount);
        renderSurpriseTable(surprises, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#e-cal');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderCalendarMatrix(events, days, mount) {
    const el = mount.querySelector('#e-cal');
    if (!el) return;
    if (!events.length) {
        el.innerHTML = `<p class="muted small">${esc(t('view.earnings_cal.hint.empty', { days }))}</p>`;
        return;
    }
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const cols = [];
    for (let i = 0; i < days; i++) {
        const d = new Date(today.getTime() + i * 86400_000);
        cols.push({ date: d, key: d.toISOString().slice(0, 10), events: [] });
    }
    for (const ev of events) {
        const c = cols.find(c => c.key === ev.earnings_date);
        if (c) c.events.push(ev);
    }
    el.innerHTML = `
        <div style="display:grid;grid-template-columns:repeat(${cols.length}, 1fr);gap:6px;">
            ${cols.map(c => `<div style="border:1px solid var(--border);padding:6px;min-height:80px;">
                <div class="small muted" style="border-bottom:1px dashed var(--border);padding-bottom:3px;margin-bottom:4px;">
                    ${dowLabel(c.date.getDay())} ${c.date.getMonth() + 1}/${c.date.getDate()}
                </div>
                ${c.events.length === 0
                    ? '<span class="muted small">—</span>'
                    : c.events.map(ev => {
                        const timing = (ev.timing || 'unknown');
                        const tag = timing === 'amc' ? '🌙' : timing === 'bmo' ? '☀' : '·';
                        const est = ev.eps_estimate != null ? t('view.earnings_cal.row.eps_estimate', { value: Number(ev.eps_estimate).toFixed(2) }) : '';
                        return `<div style="font-size:11px;padding:2px 0;">
                            <strong>${esc(ev.symbol)}</strong> ${tag}
                            <span class="muted">${esc(est)}</span>
                        </div>`;
                    }).join('')}
            </div>`).join('')}
        </div>
        <p data-i18n="view.earnings_cal.hint.before_market_open_after_market_close_timing_unkno" class="muted small" style="margin-top:6px;">☀ before market open · 🌙 after market close · · timing unknown</p>
    `;
}

function renderSurpriseTable(events, mount) {
    const el = mount.querySelector('#e-surp');
    if (!el) return;
    if (!events.length) {
        el.innerHTML = '<p data-i18n="view.earnings_cal.hint.no_surprise_data_yet" class="muted small">No surprise data yet.</p>';
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.earnings_cal.th.symbol">Symbol</th><th data-i18n="view.earnings_cal.th.date">Date</th><th data-i18n="view.earnings_cal.th.est_eps">Est EPS</th><th data-i18n="view.earnings_cal.th.actual_eps">Actual EPS</th>
            <th data-i18n="view.earnings_cal.th.surprise">Surprise</th><th data-i18n="view.earnings_cal.th.reaction_1d">Reaction 1d</th><th data-i18n="view.earnings_cal.th.reaction_5d">Reaction 5d</th>
        </tr></thead>
        <tbody>
        ${events.map(ev => {
            const sp = ev.surprise_pct;
            const r1 = ev.reaction_1d_pct;
            const r5 = ev.reaction_5d_pct;
            return `<tr>
                <td>${esc(ev.symbol)}</td>
                <td class="small">${ev.earnings_date}</td>
                <td>${ev.eps_estimate != null ? Number(ev.eps_estimate).toFixed(2) : '—'}</td>
                <td>${ev.eps_actual != null ? Number(ev.eps_actual).toFixed(2) : '—'}</td>
                <td class="${sp == null ? 'muted' : sp >= 0 ? 'pos' : 'neg'}">${sp == null ? '—' : (sp >= 0 ? '+' : '') + sp.toFixed(1) + '%'}</td>
                <td class="${r1 == null ? 'muted' : r1 >= 0 ? 'pos' : 'neg'}">${r1 == null ? '—' : (r1 >= 0 ? '+' : '') + r1.toFixed(2) + '%'}</td>
                <td class="${r5 == null ? 'muted' : r5 >= 0 ? 'pos' : 'neg'}">${r5 == null ? '—' : (r5 >= 0 ? '+' : '') + r5.toFixed(2) + '%'}</td>
            </tr>`;
        }).join('')}
        </tbody>
    </table>`;
}
