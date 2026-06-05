// Augusta Rule — IRC § 280A(g).
// Rent your home to your business for up to 14 days/year tax-free to YOU
// personally, deductible to the business. Requires documented business
// purpose (board meetings, client events, shareholder meetings, recordings).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-augusta-v1';
const MAX_DAYS = 14;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    rows: load(),
    daily_rate: 1_200,  // Comp from Airbnb/VRBO for the same space
};

export async function renderAugustaRule(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.augusta.h1.title">// AUGUSTA RULE § 280A(g)</span></h1>
        <p class="muted small" data-i18n="view.augusta.hint.intro">
            Rent your home to your business up to <strong>14 days/year tax-free</strong> to you
            personally, deductible to the business. Named after Augusta GA homeowners who rent
            during The Masters. Requires comparable Airbnb/VRBO comps + documented business purpose.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.augusta.h2.settings">Daily rate</h2>
            <form id="ar-rate-form" class="inline-form">
                <label><span data-i18n="view.augusta.label.daily_rate">Daily rate ($/day)</span>
                    <input type="number" step="0.01" name="daily_rate" value="${state.daily_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.augusta.btn.save_rate">Save rate</button>
            </form>
            <p class="muted small" data-i18n="view.augusta.rate.note">
                Comp must be reasonable — pull comps from Airbnb / VRBO / Peerspace for the same
                space + zip code. Keep screenshots dated as evidence.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.augusta.h2.add">Log meeting day</h2>
            <form id="ar-form" class="inline-form">
                <label><span data-i18n="view.augusta.label.date">Date</span>
                    <input type="date" name="date" required value="${new Date().toISOString().slice(0,10)}"></label>
                <label><span data-i18n="view.augusta.label.purpose">Business purpose</span>
                    <input type="text" name="purpose" placeholder="Q3 strategy meeting / annual shareholder meeting" required></label>
                <label><span data-i18n="view.augusta.label.attendees">Attendees</span>
                    <input type="text" name="attendees" placeholder="Self (sole shareholder) / John Doe (CPA)" required></label>
                <label><span data-i18n="view.augusta.label.minutes">Minutes documented?</span>
                    <input type="checkbox" name="minutes"></label>
                <button class="primary" type="submit" data-i18n="view.augusta.btn.add">Add</button>
            </form>
        </div>
        <div id="ar-summary"></div>
        <div id="ar-table" class="chart-panel"></div>
    `;
    document.getElementById('ar-rate-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.daily_rate = Number(fd.get('daily_rate')) || 0;
        save({ rows: state.rows, rate: state.daily_rate });
        showToast(t('view.augusta.toast.rate_saved'), { level: 'success' });
        render();
    });
    document.getElementById('ar-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const row = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            purpose: fd.get('purpose'),
            attendees: fd.get('attendees'),
            minutes: !!fd.get('minutes'),
        };
        state.rows.push(row);
        save(state.rows);
        e.target.reset();
        e.target.querySelector('[name="date"]').value = new Date().toISOString().slice(0, 10);
        showToast(t('view.augusta.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function render() {
    const year = new Date().getFullYear();
    const yearRows = state.rows.filter(r => new Date(r.date).getFullYear() === year);
    renderSummary(yearRows, year);
    renderTable(yearRows);
}

function renderSummary(yearRows, year) {
    const el = document.getElementById('ar-summary');
    if (!el) return;
    const daysUsed = yearRows.length;
    const daysRemaining = Math.max(0, MAX_DAYS - daysUsed);
    const ytdDeduction = daysUsed * state.daily_rate;
    const maxPossible = MAX_DAYS * state.daily_rate;
    const docMissing = yearRows.filter(r => !r.minutes).length;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.augusta.h2.summary">${year} usage</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.augusta.card.days_used">Days used</div>
                    <div class="value">${daysUsed} / ${MAX_DAYS}</div>
                </div>
                <div class="card ${daysRemaining > 0 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.augusta.card.days_remaining">Days remaining</div>
                    <div class="value">${daysRemaining}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.augusta.card.ytd_deduction">YTD deduction</div>
                    <div class="value">$${ytdDeduction.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.augusta.card.max_possible">Max possible (14 days)</div>
                    <div class="value">$${maxPossible.toLocaleString()}</div>
                </div>
                <div class="card ${docMissing > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.augusta.card.doc_missing">Missing minutes</div>
                    <div class="value">${docMissing}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(yearRows) {
    const el = document.getElementById('ar-table');
    if (!el) return;
    if (!yearRows.length) {
        el.innerHTML = `<h2 data-i18n="view.augusta.h2.log">Log</h2>
            <p class="muted" data-i18n="view.augusta.empty">No rental days logged this year.</p>`;
        return;
    }
    const sorted = [...yearRows].sort((a, b) => String(b.date).localeCompare(String(a.date)));
    el.innerHTML = `
        <h2 data-i18n="view.augusta.h2.log">Log</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.augusta.th.date">Date</th>
                <th data-i18n="view.augusta.th.purpose">Purpose</th>
                <th data-i18n="view.augusta.th.attendees">Attendees</th>
                <th data-i18n="view.augusta.th.minutes">Minutes</th>
                <th data-i18n="view.augusta.th.deduction">Deduction</th>
                <th data-i18n="view.augusta.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(r => `
                <tr>
                    <td>${esc(r.date)}</td>
                    <td>${esc(r.purpose)}</td>
                    <td class="muted">${esc(r.attendees)}</td>
                    <td class="${r.minutes ? 'pos' : 'neg'}">${r.minutes ? '✓' : '×'}</td>
                    <td class="pos">$${state.daily_rate.toLocaleString()}</td>
                    <td><button class="link neg" data-del="${esc(r.id)}" data-i18n="view.augusta.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
        <p class="muted small" style="margin-top:10px" data-i18n="view.augusta.disclaimer">
            Requires C-corp or S-corp business structure (sole-prop won't work — you'd be
            paying yourself). Rental income to YOU is tax-free under § 280A(g) when ≤ 14 days.
            Business deducts the rent (Schedule C / Form 1120-S). Keep written corporate
            minutes documenting business need + comparable-rate research.
        </p>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.rows = state.rows.filter(r => r.id !== btn.dataset.del);
            save(state.rows);
            render();
        });
    });
}
