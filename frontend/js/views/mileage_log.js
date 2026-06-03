// Mileage Log — log business miles trips, compute IRS deductible (67¢/mi for
// 2024, 70¢/mi for 2025). Persisted to localStorage; ready for backend port
// once a `/trips` endpoint is wired.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-mileage-trips-v1';
const RATE_2024 = 0.67;
const RATE_2025 = 0.70;
const RATE_2026 = 0.72; // IRS yearly inflation adjust; safe approximation

function currentMileageRate(year) {
    if (year >= 2026) return RATE_2026;
    if (year >= 2025) return RATE_2025;
    return RATE_2024;
}

function load() {
    try {
        const raw = localStorage.getItem(LS_KEY);
        return raw ? JSON.parse(raw) : [];
    } catch { return []; }
}
function save(trips) {
    try { localStorage.setItem(LS_KEY, JSON.stringify(trips)); } catch { /* private mode */ }
}

let state = { trips: load(), filter: 'ytd' };

export async function renderMileageLog(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mileage.h1.title">// MILEAGE LOG</span></h1>
        <p class="muted small" data-i18n="view.mileage.hint.intro">
            Log business trips, compute IRS-rate deductible miles. Persisted in this browser.
            Vehicle deduction can run into thousands per year — track every leg.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.mileage.h2.add_trip">Add trip</h2>
            <form id="ml-form" class="inline-form">
                <label><span data-i18n="view.mileage.label.date">Date</span>
                    <input type="date" name="date" required value="${new Date().toISOString().slice(0,10)}"></label>
                <label><span data-i18n="view.mileage.label.from">From</span>
                    <input type="text" name="from_place" placeholder="Home" required></label>
                <label><span data-i18n="view.mileage.label.to">To</span>
                    <input type="text" name="to_place" placeholder="Client / Bank / Conference" required></label>
                <label><span data-i18n="view.mileage.label.miles">Miles</span>
                    <input type="number" step="0.1" name="miles" required min="0.1"></label>
                <label><span data-i18n="view.mileage.label.purpose">Purpose</span>
                    <input type="text" name="purpose" placeholder="Client meeting / Bank deposit / Supplies" required></label>
                <label><span data-i18n="view.mileage.label.return_trip">Round trip?</span>
                    <input type="checkbox" name="round_trip" checked></label>
                <button class="primary" type="submit" data-i18n="view.mileage.btn.add">Add</button>
            </form>
        </div>
        <div id="ml-summary" class="chart-panel"></div>
        <div id="ml-table" class="chart-panel"></div>
    `;
    document.getElementById('ml-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const miles = Number(fd.get('miles'));
        const round = !!fd.get('round_trip');
        const trip = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            from_place: fd.get('from_place'),
            to_place: fd.get('to_place'),
            miles: round ? miles * 2 : miles,
            purpose: fd.get('purpose'),
            round_trip: round,
            created_at: new Date().toISOString(),
        };
        state.trips.unshift(trip);
        save(state.trips);
        e.target.reset();
        showToast(t('view.mileage.toast.added', { miles: trip.miles.toFixed(1) }), { level: 'success' });
        render();
    });
    render();
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('ml-summary');
    if (!el) return;
    const now = new Date();
    const ytdStart = new Date(now.getFullYear(), 0, 1);
    const ytdTrips = state.trips.filter(tr => new Date(tr.date) >= ytdStart);
    const ytdMiles = ytdTrips.reduce((s, t) => s + Number(t.miles || 0), 0);
    const rate = currentMileageRate(now.getFullYear());
    const ytdDeduction = ytdMiles * rate;
    const monthMiles = state.trips
        .filter(tr => {
            const d = new Date(tr.date);
            return d.getFullYear() === now.getFullYear() && d.getMonth() === now.getMonth();
        })
        .reduce((s, t) => s + Number(t.miles || 0), 0);
    const monthDeduction = monthMiles * rate;
    el.innerHTML = `
        <h2 data-i18n="view.mileage.h2.summary">YTD summary</h2>
        <div class="cards">
            <div class="card">
                <div class="label" data-i18n="view.mileage.card.ytd_miles">YTD miles</div>
                <div class="value">${ytdMiles.toLocaleString(undefined, { maximumFractionDigits: 1 })}</div>
            </div>
            <div class="card pos">
                <div class="label" data-i18n="view.mileage.card.ytd_deduction">YTD deductible</div>
                <div class="value">$${ytdDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.mileage.card.month_miles">This month</div>
                <div class="value">${monthMiles.toLocaleString(undefined, { maximumFractionDigits: 1 })} mi</div>
            </div>
            <div class="card pos">
                <div class="label" data-i18n="view.mileage.card.month_deduction">This month deductible</div>
                <div class="value">$${monthDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.mileage.card.trip_count">Trip count</div>
                <div class="value">${ytdTrips.length}</div>
            </div>
            <div class="card">
                <div class="label" data-i18n="view.mileage.card.rate">IRS rate</div>
                <div class="value">${(rate * 100).toFixed(0)}¢/mi</div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('ml-table');
    if (!el) return;
    const sorted = [...state.trips].sort((a, b) => String(b.date).localeCompare(String(a.date)));
    if (!sorted.length) {
        el.innerHTML = `<h2 data-i18n="view.mileage.h2.log">Trip log</h2>
            <p class="muted" data-i18n="view.mileage.empty">No trips logged yet.</p>`;
        return;
    }
    el.innerHTML = `
        <h2 data-i18n="view.mileage.h2.log">Trip log</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.mileage.th.date">Date</th>
                <th data-i18n="view.mileage.th.from">From</th>
                <th data-i18n="view.mileage.th.to">To</th>
                <th data-i18n="view.mileage.th.miles">Miles</th>
                <th data-i18n="view.mileage.th.purpose">Purpose</th>
                <th data-i18n="view.mileage.th.deduction">Deduction</th>
                <th data-i18n="view.mileage.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(tr => {
                const year = new Date(tr.date).getFullYear();
                const rate = currentMileageRate(year);
                const ded = Number(tr.miles || 0) * rate;
                return `<tr>
                    <td>${esc(tr.date)}</td>
                    <td>${esc(tr.from_place || '')}</td>
                    <td>${esc(tr.to_place || '')}</td>
                    <td>${Number(tr.miles || 0).toFixed(1)}${tr.round_trip ? ' <span class="muted small">↔</span>' : ''}</td>
                    <td class="muted">${esc(tr.purpose || '')}</td>
                    <td class="pos">$${ded.toFixed(2)}</td>
                    <td><button class="link neg" data-del="${esc(tr.id)}" data-i18n="view.mileage.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            const id = btn.dataset.del;
            state.trips = state.trips.filter(t => t.id !== id);
            save(state.trips);
            render();
        });
    });
}
