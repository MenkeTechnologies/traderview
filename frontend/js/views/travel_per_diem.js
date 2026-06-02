// Travel Per Diem Tracker — IRS / GSA per-diem rates.
// Default approach uses CONUS standard rate ($178/day = $107 lodging + $71 M&IE
// for FY2025). Major city rates higher. Each trip log captures both lodging
// actual + per-diem M&IE so you pick the higher.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-travel-v1';

// FY2025 GSA per-diem rates ($/day) — sampling of major cities.
// "M&IE" = meals & incidentals. Lodging shown is "max", actual rate
// varies by month. CONUS-standard is the floor for non-listed.
const PER_DIEMS = {
    'CONUS-STANDARD':    { lodging: 110, mie: 73 },
    'New York, NY':      { lodging: 379, mie: 92 },
    'San Francisco, CA': { lodging: 327, mie: 92 },
    'Chicago, IL':       { lodging: 270, mie: 92 },
    'Boston, MA':        { lodging: 285, mie: 92 },
    'Washington, DC':    { lodging: 274, mie: 92 },
    'Los Angeles, CA':   { lodging: 217, mie: 86 },
    'Seattle, WA':       { lodging: 254, mie: 92 },
    'Denver, CO':        { lodging: 209, mie: 86 },
    'Las Vegas, NV':     { lodging: 195, mie: 80 },
    'Miami, FL':         { lodging: 228, mie: 86 },
    'Honolulu, HI':      { lodging: 256, mie: 92 },
};
const FIRST_LAST_DAY_FACTOR = 0.75; // M&IE is 75% on travel day

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(trips) { try { localStorage.setItem(LS_KEY, JSON.stringify(trips)); } catch { /* ignore */ } }

let state = { trips: load() };

export async function renderTravelPerDiem(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.travel.h1.title">// TRAVEL PER-DIEM TRACKER</span></h1>
        <p class="muted small" data-i18n="view.travel.hint.intro">
            Log business trips with destination + nights + lodging actual + per-diem
            M&amp;IE. Per-diem M&amp;IE is 75% on travel days. Pick the higher of
            actual receipts or per-diem on Schedule C line 24a (travel).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.travel.h2.add">Add trip</h2>
            <form id="tv-form" class="inline-form">
                <label><span data-i18n="view.travel.label.start_date">Start date</span>
                    <input type="date" name="start_date" required value="${new Date().toISOString().slice(0,10)}"></label>
                <label><span data-i18n="view.travel.label.end_date">End date</span>
                    <input type="date" name="end_date" required></label>
                <label><span data-i18n="view.travel.label.destination">Destination</span>
                    <select name="destination">${Object.keys(PER_DIEMS).map(d =>
                        `<option value="${esc(d)}">${esc(d)}</option>`
                    ).join('')}</select>
                </label>
                <label><span data-i18n="view.travel.label.lodging_actual">Lodging actual ($)</span>
                    <input type="number" step="0.01" name="lodging_actual" value="0"></label>
                <label><span data-i18n="view.travel.label.airfare">Airfare ($)</span>
                    <input type="number" step="0.01" name="airfare" value="0"></label>
                <label><span data-i18n="view.travel.label.transport">Ground transport ($)</span>
                    <input type="number" step="0.01" name="ground_transport" value="0"></label>
                <label><span data-i18n="view.travel.label.purpose">Business purpose</span>
                    <input type="text" name="purpose" placeholder="Conference / client meeting / training" required></label>
                <button class="primary" type="submit" data-i18n="view.travel.btn.add">Add</button>
            </form>
        </div>
        <div id="tv-summary"></div>
        <div id="tv-table" class="chart-panel"></div>
    `;
    document.getElementById('tv-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const trip = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            start_date: fd.get('start_date'),
            end_date: fd.get('end_date'),
            destination: fd.get('destination'),
            lodging_actual: Number(fd.get('lodging_actual')) || 0,
            airfare: Number(fd.get('airfare')) || 0,
            ground_transport: Number(fd.get('ground_transport')) || 0,
            purpose: fd.get('purpose'),
        };
        state.trips.push(trip);
        save(state.trips);
        e.target.reset();
        e.target.querySelector('[name="start_date"]').value = new Date().toISOString().slice(0, 10);
        showToast(t('view.travel.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function nightsFromTrip(t) {
    const s = new Date(t.start_date);
    const e = new Date(t.end_date);
    return Math.max(0, Math.round((e - s) / 86_400_000));
}

function fullDays(nights) {
    // For an N-night trip, M&IE: 2 travel days × 75% + (N-1) full days at 100%.
    // Use max(0, ...) for single-day no-overnight trips.
    return { fullDays: Math.max(0, nights - 1), travelDays: nights > 0 ? 2 : 0 };
}

function computeMIE(t) {
    const rates = PER_DIEMS[t.destination] || PER_DIEMS['CONUS-STANDARD'];
    const nights = nightsFromTrip(t);
    const { fullDays: fd, travelDays } = fullDays(nights);
    return fd * rates.mie + travelDays * rates.mie * FIRST_LAST_DAY_FACTOR;
}
function computeLodgingPerDiem(t) {
    const rates = PER_DIEMS[t.destination] || PER_DIEMS['CONUS-STANDARD'];
    const nights = nightsFromTrip(t);
    return nights * rates.lodging;
}
function tripDeductible(t) {
    const lodging = Math.max(t.lodging_actual, computeLodgingPerDiem(t));
    const mie = computeMIE(t);
    return {
        lodging, mie,
        meals_deductible: mie * 0.5,  // M&IE 50% deductible
        airfare: t.airfare, ground: t.ground_transport,
        total: lodging + (mie * 0.5) + t.airfare + t.ground_transport,
    };
}

function render() {
    const summary = document.getElementById('tv-summary');
    const tableEl = document.getElementById('tv-table');
    if (summary) renderSummary(summary);
    if (tableEl) renderTable(tableEl);
}

function renderSummary(el) {
    const year = new Date().getFullYear();
    const yearTrips = state.trips.filter(t => new Date(t.start_date).getFullYear() === year);
    const totals = yearTrips.reduce((acc, t) => {
        const d = tripDeductible(t);
        acc.lodging += d.lodging;
        acc.meals += d.meals_deductible;
        acc.airfare += d.airfare;
        acc.ground += d.ground;
        acc.total += d.total;
        return acc;
    }, { lodging: 0, meals: 0, airfare: 0, ground: 0, total: 0 });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.travel.h2.summary">${year} totals</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.travel.card.trips">Trips</div>
                    <div class="value">${yearTrips.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.travel.card.lodging">Lodging</div>
                    <div class="value">$${totals.lodging.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.travel.card.meals">Meals (50%)</div>
                    <div class="value">$${totals.meals.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.travel.card.airfare">Airfare</div>
                    <div class="value">$${totals.airfare.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.travel.card.ground">Ground</div>
                    <div class="value">$${totals.ground.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.travel.card.total">Total deductible</div>
                    <div class="value">$${totals.total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(el) {
    if (!state.trips.length) {
        el.innerHTML = `<h2 data-i18n="view.travel.h2.log">Log</h2>
            <p class="muted" data-i18n="view.travel.empty">No trips yet.</p>`;
        return;
    }
    const sorted = [...state.trips].sort((a, b) => String(b.start_date).localeCompare(String(a.start_date)));
    el.innerHTML = `
        <h2 data-i18n="view.travel.h2.log">Log</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.travel.th.dates">Dates</th>
                <th data-i18n="view.travel.th.destination">Destination</th>
                <th data-i18n="view.travel.th.nights">Nights</th>
                <th data-i18n="view.travel.th.lodging_actual">Lodging actual</th>
                <th data-i18n="view.travel.th.lodging_perdiem">Lodging per-diem</th>
                <th data-i18n="view.travel.th.mie">M&IE</th>
                <th data-i18n="view.travel.th.deductible">Deductible</th>
                <th data-i18n="view.travel.th.purpose">Purpose</th>
                <th data-i18n="view.travel.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(tr => {
                const nights = nightsFromTrip(tr);
                const d = tripDeductible(tr);
                const lodgingPD = computeLodgingPerDiem(tr);
                return `<tr>
                    <td class="muted">${esc(tr.start_date)} → ${esc(tr.end_date)}</td>
                    <td>${esc(tr.destination)}</td>
                    <td>${nights}</td>
                    <td>$${tr.lodging_actual.toFixed(0)}</td>
                    <td class="muted">$${lodgingPD.toFixed(0)}</td>
                    <td>$${d.mie.toFixed(0)}</td>
                    <td class="pos">$${d.total.toFixed(0)}</td>
                    <td class="muted">${esc((tr.purpose || '').slice(0, 50))}</td>
                    <td><button class="link neg" data-del="${esc(tr.id)}" data-i18n="view.travel.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.trips = state.trips.filter(t => t.id !== btn.dataset.del);
            save(state.trips);
            render();
        });
    });
}
