// Up / down capture ratio — gains captured in up markets vs losses in down,
// via /calc/capture-ratio. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const ratio = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));

function parseSeries(raw) {
    return String(raw || '')
        .split(/[\s,]+/)
        .map((s) => s.trim())
        .filter((s) => s.length)
        .map(Number)
        .filter((n) => Number.isFinite(n));
}

export async function renderCaptureRatio(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.capture.h1.title">// UP / DOWN CAPTURE RATIO</span></h1>
        <p class="muted small" data-i18n="view.capture.hint.intro">
            Split periods by the benchmark's sign, geometrically compound each side, and compare. Up
            capture above 100 means the strategy beats the benchmark in rising markets; down capture
            below 100 means it loses less when the benchmark falls. A capture ratio above 1 is the
            favorable asymmetry. Enter paired period returns in percent. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.capture.h2.inputs">The returns</h2>
            <form id="capture-form" class="inline-form">
                <label><span data-i18n="view.capture.label.fund">Strategy returns (%, comma-separated)</span>
                    <input type="text" name="fund_returns_pct" value="20, -5, 20, -5" required></label>
                <label><span data-i18n="view.capture.label.bench">Benchmark returns (%, comma-separated)</span>
                    <input type="text" name="benchmark_returns_pct" value="10, -10, 10, -10" required></label>
            </form>
        </div>
        <div id="capture-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#capture-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            fund_returns_pct: parseSeries(fd.get('fund_returns_pct')),
            benchmark_returns_pct: parseSeries(fd.get('benchmark_returns_pct')),
        };
        try {
            const r = await api.calcCaptureRatio(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.capture.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#capture-result');
    const cls = r.is_favorable ? 'pos' : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.capture.h2.result">The capture</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.capture.card.up">Up capture</div>
                    <div class="value pos">${pct(r.up_capture_pct)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.capture.card.down">Down capture</div>
                    <div class="value neg">${pct(r.down_capture_pct)}</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.capture.card.ratio">Capture ratio</div>
                    <div class="value ${cls}">${ratio(r.capture_ratio)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.capture.row.up_periods">Up periods</td><td>${r.up_periods}</td></tr>
                    <tr><td data-i18n="view.capture.row.down_periods">Down periods</td><td>${r.down_periods}</td></tr>
                    <tr><td data-i18n="view.capture.row.fund_up">Strategy in up markets</td><td>${pct(r.fund_up_return_pct)}</td></tr>
                    <tr><td data-i18n="view.capture.row.bench_up">Benchmark in up markets</td><td>${pct(r.benchmark_up_return_pct)}</td></tr>
                    <tr><td data-i18n="view.capture.row.fund_down">Strategy in down markets</td><td>${pct(r.fund_down_return_pct)}</td></tr>
                    <tr><td data-i18n="view.capture.row.bench_down">Benchmark in down markets</td><td>${pct(r.benchmark_down_return_pct)}</td></tr>
                    <tr><td data-i18n="view.capture.row.up_capture">Up capture</td><td>${pct(r.up_capture_pct)}</td></tr>
                    <tr><td data-i18n="view.capture.row.down_capture">Down capture</td><td>${pct(r.down_capture_pct)}</td></tr>
                    <tr class="emph ${cls}"><td data-i18n="view.capture.row.ratio">Capture ratio</td><td>${ratio(r.capture_ratio)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
