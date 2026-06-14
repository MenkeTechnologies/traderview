// Roth IRA contribution limit after the MAGI phase-out, via
// /calc/roth-contribution. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });

const STATUS_CLS = { full: 'pos', partial: '', none: 'neg' };
const VIEW = 'roth-contribution';
let lastReport = null;
let lastBody = null;

export async function renderRothContribution(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rothlimit.h1.title">// ROTH IRA LIMIT</span></h1>
        <p class="muted small" data-i18n="view.rothlimit.hint.intro">
            Above a modified-AGI threshold your Roth IRA contribution phases down to zero. Enter your
            MAGI and filing status to see what you can still contribute. 2026 limits: $7,500 base,
            +$1,100 catch-up at 50+; phase-out $153k–$168k single, $242k–$252k married-joint,
            $0–$10k married-separate. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rothlimit.h2.inputs">Your situation</h2>
            <form id="roth-form" class="inline-form">
                <label><span data-i18n="view.rothlimit.label.magi">Modified AGI ($)</span>
                    <input type="number" step="0.01" min="0" name="magi_usd" value="160000" required></label>
                <label><span data-i18n="view.rothlimit.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" data-i18n="view.rothlimit.status.single">Single / HoH</option>
                        <option value="married_joint" data-i18n="view.rothlimit.status.mfj">Married filing jointly</option>
                        <option value="married_separate" data-i18n="view.rothlimit.status.mfs">Married filing separately</option>
                    </select></label>
                <label><span data-i18n="view.rothlimit.label.age50">Age 50 or older (catch-up)</span>
                    <input type="checkbox" name="age_50_plus"></label>
                <label><span data-i18n="view.rothlimit.label.base">Base limit ($)</span>
                    <input type="number" step="1" min="0" name="base_limit_usd" value="7500"></label>
                <label><span data-i18n="view.rothlimit.label.catchup">Catch-up ($)</span>
                    <input type="number" step="1" min="0" name="catch_up_usd" value="1100"></label>
            </form>
            <div id="roth-tools" class="ce-toolbar"></div>
        </div>
        <div id="roth-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#roth-form');
    const hashIn = enh.readHashInputs();
    enh.prefillForm(form, hashIn);
    if ('age_50_plus' in hashIn) form.querySelector('[name=age_50_plus]').checked = hashIn.age_50_plus === 'true';
    const readBody = () => {
        const fd = new FormData(form);
        return {
            magi_usd: Number(fd.get('magi_usd')) || 0,
            filing_status: fd.get('filing_status'),
            age_50_plus: form.querySelector('[name=age_50_plus]').checked,
            base_limit_usd: Number(fd.get('base_limit_usd')) || 0,
            catch_up_usd: Number(fd.get('catch_up_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcRothContribution(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.rothlimit.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#roth-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'roth-contribution.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['allowed_contribution_usd', r.allowed_contribution_usd],
        ['max_contribution_usd', r.max_contribution_usd],
        ['disallowed_usd', r.disallowed_usd],
        ['phaseout_start_usd', r.phaseout_start_usd],
        ['phaseout_end_usd', r.phaseout_end_usd],
        ['status', r.status],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#roth-result');
    const cls = STATUS_CLS[r.status] ?? '';
    // Line chart: allowed contribution across MAGI 0 -> $300k (the phase-out ramp to zero).
    const xs = enh.linspace(0, 300000, 16);
    const pts = await Promise.all(xs.map(async (m) => {
        const rr = await api.calcRothContribution({ ...body, magi_usd: m });
        return { x: m / 1000, y: rr ? rr.allowed_contribution_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'MAGI $k', ylabel: 'allowed $' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rothlimit.h2.result">What you can contribute</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="view.rothlimit.card.allowed">Allowed</div>
                    <div class="value ${cls}">${money(r.allowed_contribution_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rothlimit.card.max">Max limit</div>
                    <div class="value">${money(r.max_contribution_usd)}</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.rothlimit.card.status">Status</div>
                    <div class="value ${cls}" data-i18n="view.rothlimit.status.${r.status}">—</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.rothlimit.row.max">Max limit</td><td>${money(r.max_contribution_usd)}</td></tr>
                    <tr><td data-i18n="view.rothlimit.row.disallowed">Phased out</td><td>${money(r.disallowed_usd)}</td></tr>
                    <tr><td data-i18n="view.rothlimit.row.start">Phase-out start</td><td>${money(r.phaseout_start_usd)}</td></tr>
                    <tr><td data-i18n="view.rothlimit.row.end">Phase-out end</td><td>${money(r.phaseout_end_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.rothlimit.row.allowed">Allowed contribution</td><td>${money(r.allowed_contribution_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
