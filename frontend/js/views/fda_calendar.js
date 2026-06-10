// FDA Advisory Committee Calendar — Finnhub /fda-advisory-committee-calendar.
// Biotech catalyst gold: PDUFA dates + ad-com meetings.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { searchScore, getMatchIndices, highlightWithIndices } from '../fzf.js';

const state = { rows: [], filter: '' };

export async function renderFdaCalendar(mount, _appState) {
    const tok = currentViewToken();
    state.filter = '';
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.fda_calendar.h1.title">// FDA ADVISORY CALENDAR</span></h1>
        <p class="muted small" data-i18n="view.fda_calendar.hint.intro">
            Upcoming FDA advisory committee meetings + PDUFA dates. Biotech runners
            often originate here — track ad-com votes for sympathy plays.
        </p>
        <div class="chart-panel">
            <div class="inline-form">
                <button class="primary" id="fda-refresh" type="button" data-i18n="view.fda_calendar.btn.refresh">Refresh</button>
                <label><span data-i18n="view.fda_calendar.label.filter">Filter</span>
                    <input id="fda-filter" type="text" placeholder="committee / drug / date"
                        data-i18n-placeholder="view.fda_calendar.placeholder.filter"
                        value="${esc(state.filter)}"></label>
            </div>
            <div id="fda-table" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('fda-refresh').addEventListener('click', () => void load(tok));
    document.getElementById('fda-filter').addEventListener('input', e => {
        state.filter = e.target.value;
        renderTable();
    });
    await load(tok);
}

// Finnhub's FDAComitteeMeeting only exposes: eventDescription, fromDate,
// toDate, url. There is no structured committee/date field, so derive a clean
// date from fromDate and best-effort the committee name out of the description.
function fmtDate(s) {
    if (!s) return '';
    const m = String(s).trim().match(/^(\d{4})-(\d{2})-(\d{2})/);
    return m ? `${m[2]}/${m[3]}/${m[1]}` : String(s).trim();
}
function fmtDateTime(s) {
    if (!s) return '';
    const str = String(s).trim();
    const time = str.match(/[ T](\d{2}:\d{2})(?::\d{2})?/);
    const date = fmtDate(str);
    return time && time[1] !== '00:00' ? `${date} ${time[1]}` : date;
}
function deriveCommittee(desc) {
    if (!desc) return '';
    const tokens = String(desc).split(/\s+/);
    let end = -1;
    for (let i = tokens.length - 1; i >= 0; i--) {
        const w = tokens[i].replace(/[^A-Za-z]/g, '');
        if (w === 'Committee' || w === 'Panel') { end = i; break; }
    }
    if (end < 0) return '';
    const connectors = new Set(['and', 'of', 'the', 'for', 'on', 'to', '&']);
    const isWord = (tk) => {
        const w = tk.replace(/[.,]+$/, '');
        if (connectors.has(w.toLowerCase())) return true;
        // Title-case word (has a lowercase letter, no digits) — skips ALL-CAPS
        // status banners ("UPDATED", "POSTPONED") and dates.
        return /^[A-Z][A-Za-z'&/-]*$/.test(w) && /[a-z]/.test(w);
    };
    let start = end;
    while (start - 1 >= 0 && isWord(tokens[start - 1])) start--;
    let phrase = tokens.slice(start, end + 1).join(' ').replace(/[.,]+$/, '');
    phrase = phrase
        .replace(/^((?:Joint\s+)?Meeting\s+)?(?:of\s+|the\s+|for\s+)*/i, '')
        .replace(/\s+/g, ' ')
        .trim();
    return phrase;
}

async function load(tok) {
    const el = document.getElementById('fda-table');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.finnhubFdaCalendar();
        if (!viewIsCurrent(tok)) return;
        const rows = Array.isArray(data) ? data : (data?.data || []);
        state.rows = [...rows].sort((a, b) =>
            String(b.fromDate || '').localeCompare(String(a.fromDate || '')));
        renderTable();
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.fda_calendar.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.fda_calendar.toast.failed'), { level: 'error' });
    }
}

function renderTable() {
    const el = document.getElementById('fda-table');
    if (!el) return;
    if (!state.rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.fda_calendar.empty">No upcoming FDA meetings.</p>`;
        return;
    }
    const enriched = state.rows.map(r => {
        const desc = r.eventDescription || r.eventName || '';
        return { r, desc, committee: deriveCommittee(desc) };
    });
    const q = state.filter.trim();
    const ranked = q
        ? enriched
            .map(row => {
                const { r, desc, committee } = row;
                const fields = [desc, committee, fmtDate(r.fromDate), fmtDate(r.toDate)];
                const score = searchScore(q, fields);
                return score > 0 ? { row, score } : null;
            })
            .filter(Boolean)
            .sort((a, b) => b.score - a.score)
        : enriched.map(row => ({ row, score: 0 }));
    if (!ranked.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.fda_calendar.no_match">No meetings match the filter.</p>`;
        return;
    }
    const hl = (text) => q ? highlightWithIndices(text, getMatchIndices(q, text)) : esc(text);
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.fda_calendar.th.event_date">Event date</th>
            <th data-i18n="view.fda_calendar.th.event">Event / drug</th>
            <th data-i18n="view.fda_calendar.th.committee">Committee</th>
            <th data-i18n="view.fda_calendar.th.start">Start</th>
            <th data-i18n="view.fda_calendar.th.end">End</th>
        </tr></thead>
        <tbody>${ranked.map(({ row: { r, desc, committee } }) => {
            const descText = desc || '—';
            const eventCell = r.url
                ? `<a class="link" href="${esc(r.url)}" target="_blank" rel="noopener">${hl(descText)}</a>`
                : hl(descText);
            return `
            <tr>
                <td><strong>${esc(fmtDate(r.fromDate) || '—')}</strong></td>
                <td>${eventCell}</td>
                <td class="muted">${hl(committee || '—')}</td>
                <td class="muted">${esc(fmtDateTime(r.fromDate) || '—')}</td>
                <td class="muted">${esc(fmtDateTime(r.toDate) || '—')}</td>
            </tr>`;
        }).join('')}</tbody>
    </table>`;
}
