// Shared calculator enhancements — charts, export, sensitivity, and permalinks.
//
// These helpers are config-driven so every calculator view wires them the same
// way instead of re-implementing one-off chart/export/sensitivity code. A view
// passes its inputs, a compute callback, and field metadata; the module renders
// the toolbar, an SVG chart, and an optional two-axis sensitivity grid.
//
// SVG colors are hardcoded (not CSS custom properties) because release-WebKit
// does not reliably resolve var() on dynamically inserted SVG. All charts use
// plain strokes/fills only — no box-shadow, animation, or pseudo-elements —
// which are the constructs that corrupt rendering in release builds.
import { t } from './i18n.js';
import { showToast } from './toast.js';
import { esc } from './util.js';

const CYAN = '#00e5ff';
const GREEN = '#39ff14';
const RED = '#ff3b5c';
const DIM = '#5a6b78';

// ---- SVG charts ---------------------------------------------------------

// Line chart from numeric points [{x, y}]. Returns an SVG markup string (or ''
// when there is nothing plottable).
export function svgLineChart(points, opts = {}) {
    const pts = (points || []).filter((p) => Number.isFinite(p.x) && Number.isFinite(p.y));
    if (pts.length < 2) return '';
    const { w = 340, h = 150, pad = 28, stroke = CYAN, xlabel = '', ylabel = '' } = opts;
    const xs = pts.map((p) => p.x);
    const ys = pts.map((p) => p.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys, 0);
    const maxY = Math.max(...ys);
    const sx = (v) => pad + (v - minX) / (maxX - minX || 1) * (w - 2 * pad);
    const sy = (v) => h - pad - (v - minY) / (maxY - minY || 1) * (h - 2 * pad);
    const poly = pts.map((p) => `${sx(p.x).toFixed(1)},${sy(p.y).toFixed(1)}`).join(' ');
    const zeroY = (minY < 0 && maxY > 0) ? `<line x1="${pad}" y1="${sy(0).toFixed(1)}" x2="${w - pad}" y2="${sy(0).toFixed(1)}" stroke="${DIM}" stroke-width="0.5" stroke-dasharray="3 3"></line>` : '';
    return `<svg viewBox="0 0 ${w} ${h}" class="ce-svg" role="img" aria-label="${esc(xlabel)} chart" preserveAspectRatio="xMidYMid meet">
        ${zeroY}
        <polyline points="${poly}" fill="none" stroke="${stroke}" stroke-width="1.5"></polyline>
        <text x="${pad}" y="${h - 6}" class="ce-axis">${esc(xlabel)} →</text>
        <text x="6" y="${pad - 8}" class="ce-axis">${esc(ylabel)} ↑</text>
    </svg>`;
}

// Vertical bar chart from [{label, value}]. Bars are green when positive, red
// when negative, so sign reads at a glance.
export function svgBarChart(bars, opts = {}) {
    const data = (bars || []).filter((b) => Number.isFinite(b.value));
    if (!data.length) return '';
    const { w = 340, h = 150, pad = 28 } = opts;
    const maxAbs = Math.max(...data.map((b) => Math.abs(b.value)), 1e-9);
    const innerH = h - 2 * pad;
    const baseY = pad + innerH / 2;
    const slot = (w - 2 * pad) / data.length;
    const bw = Math.max(4, slot * 0.6);
    const cells = data.map((b, i) => {
        const cx = pad + slot * i + slot / 2;
        const barH = Math.abs(b.value) / maxAbs * (innerH / 2);
        const y = b.value >= 0 ? baseY - barH : baseY;
        const fill = b.value >= 0 ? GREEN : RED;
        return `<rect x="${(cx - bw / 2).toFixed(1)}" y="${y.toFixed(1)}" width="${bw.toFixed(1)}" height="${barH.toFixed(1)}" fill="${fill}"></rect>
            <text x="${cx.toFixed(1)}" y="${h - 8}" class="ce-axis" text-anchor="middle">${esc(String(b.label).slice(0, 8))}</text>`;
    }).join('');
    return `<svg viewBox="0 0 ${w} ${h}" class="ce-svg" role="img" aria-label="bar chart" preserveAspectRatio="xMidYMid meet">
        <line x1="${pad}" y1="${baseY.toFixed(1)}" x2="${w - pad}" y2="${baseY.toFixed(1)}" stroke="${DIM}" stroke-width="0.5"></line>
        ${cells}
    </svg>`;
}

// ---- Export -------------------------------------------------------------

// Serialize a rows matrix (array of arrays) to CSV text, quoting cells that
// contain a comma, quote, or newline.
export function toCsv(rows) {
    return (rows || []).map((r) => r.map((cell) => {
        const s = cell == null ? '' : String(cell);
        return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
    }).join(',')).join('\n');
}

export function copyText(text, whatLabel) {
    if (!navigator.clipboard || !navigator.clipboard.writeText) {
        showToast(t('toast.err.clipboard_denied') || 'Clipboard unavailable', { level: 'error' });
        return;
    }
    void navigator.clipboard.writeText(text).then(
        () => showToast(t('calc.enh.copied', { what: whatLabel || '' }) || 'Copied'),
        () => showToast(t('toast.err.clipboard_denied') || 'Clipboard denied', { level: 'error' }),
    );
}

function downloadCsv(filename, rows) {
    const blob = new Blob([toCsv(rows)], { type: 'text/csv;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
}

// ---- Permalinks ---------------------------------------------------------

// Build a shareable URL that encodes the current inputs as query params on the
// view's hash route — e.g. `…/#saas-magic-number?prior_quarter_revenue_usd=1000000`.
export function buildPermalink(viewId, inputs) {
    const qs = new URLSearchParams();
    Object.entries(inputs || {}).forEach(([k, v]) => {
        if (v != null && v !== '') qs.set(k, String(v));
    });
    const base = location.origin + location.pathname;
    const q = qs.toString();
    return `${base}#${viewId}${q ? `?${q}` : ''}`;
}

// Read query params off the current hash route into a plain object.
export function readHashInputs() {
    const h = (location.hash || '').slice(1);
    const q = h.indexOf('?');
    if (q < 0) return {};
    return Object.fromEntries(new URLSearchParams(h.slice(q + 1)));
}

// Prefill a form's named inputs from a params object (string values, as URL
// params arrive). Unknown keys are ignored.
export function prefillForm(form, params) {
    if (!form || !params) return;
    Object.entries(params).forEach(([k, v]) => {
        const el = form.querySelector(`[name="${CSS.escape(k)}"]`);
        if (el) el.value = v;
    });
}

// ---- Toolbar widget -----------------------------------------------------

// Render a Copy / CSV / Link toolbar into `container`. Callbacks are invoked at
// click time so they always reflect the latest computed result.
//   getRows()   → array-of-arrays for CSV + clipboard (header row first)
//   getInputs() → current input object for the permalink
//   link        → set false to omit the Link button for views that cannot
//                 prefill from the hash (e.g. id-based, non-name forms)
export function mountToolbar(container, { viewId, getInputs, getRows, filename, link = true }) {
    if (!container) return;
    const linkBtn = link ? `<button type="button" class="ce-tool" data-act="link">${t('calc.enh.link') || '🔗 Link'}</button>` : '';
    container.innerHTML = `
        <button type="button" class="ce-tool" data-act="copy">${t('calc.enh.copy') || '📋 Copy'}</button>
        <button type="button" class="ce-tool" data-act="csv">${t('calc.enh.csv') || '⬇ CSV'}</button>
        ${linkBtn}`;
    container.querySelector('[data-act="copy"]').addEventListener('click', () => {
        const rows = (getRows && getRows()) || [];
        copyText(rows.map((r) => r.join('\t')).join('\n'), t('calc.enh.what.result') || 'result');
    });
    container.querySelector('[data-act="csv"]').addEventListener('click', () => {
        const rows = (getRows && getRows()) || [];
        if (!rows.length) return;
        downloadCsv(filename || `${viewId}.csv`, rows);
    });
    if (link) {
        container.querySelector('[data-act="link"]').addEventListener('click', () => {
            copyText(buildPermalink(viewId, (getInputs && getInputs()) || {}), t('calc.enh.what.link') || 'link');
        });
    }
}

// ---- Sensitivity --------------------------------------------------------

// Build an inclusive numeric range of `steps` values from lo to hi.
export function linspace(lo, hi, steps) {
    if (steps <= 1) return [lo];
    const out = [];
    for (let i = 0; i < steps; i++) out.push(lo + (hi - lo) * (i / (steps - 1)));
    return out;
}

// Compute a two-axis sensitivity matrix. For each (yVal, xVal) it clones `base`,
// overrides xKey/yKey, calls `compute(body)` (async), and extracts a scalar via
// `pick(result)`. Returns { xVals, yVals, cells } where cells[row][col] is the
// scalar (or null on failure).
export async function runSensitivity({ base, xKey, yKey, xVals, yVals, compute, pick }) {
    const cells = [];
    for (const yv of yVals) {
        const row = [];
        for (const xv of xVals) {
            const body = { ...base, [xKey]: xv, [yKey]: yv };
            try {
                const r = await compute(body);
                const v = pick(r);
                row.push(Number.isFinite(v) ? v : null);
            } catch (_) {
                row.push(null);
            }
        }
        cells.push(row);
    }
    return { xVals, yVals, cells };
}

// Render a sensitivity matrix as an HTML table with heatmap shading (green for
// high, red for low, relative to the matrix min/max). `fmt` formats each scalar;
// `xfmt`/`yfmt` format the axis headers.
export function renderSensitivityTable({ xVals, yVals, cells, fmt, xfmt, yfmt, xName, yName }) {
    const flat = cells.flat().filter((v) => v != null);
    const lo = flat.length ? Math.min(...flat) : 0;
    const hi = flat.length ? Math.max(...flat) : 1;
    const shade = (v) => {
        if (v == null) return '';
        const f = hi === lo ? 0.5 : (v - lo) / (hi - lo);
        // Interpolate red→green through the f in [0,1].
        const r = Math.round(255 * (1 - f) + 57 * f);
        const g = Math.round(59 * (1 - f) + 255 * f);
        const b = Math.round(92 * (1 - f) + 20 * f);
        return ` style="background:rgba(${r},${g},${b},0.18)"`;
    };
    const f = fmt || ((v) => (v == null ? '—' : v.toFixed(2)));
    const xf = xfmt || ((v) => v.toFixed(0));
    const yf = yfmt || ((v) => v.toFixed(0));
    const head = `<tr><th class="ce-sens-corner">${esc(yName || '')} \\ ${esc(xName || '')}</th>${xVals.map((x) => `<th>${esc(xf(x))}</th>`).join('')}</tr>`;
    const body = yVals.map((y, ri) => `<tr><th>${esc(yf(y))}</th>${cells[ri].map((v) => `<td${shade(v)}>${esc(f(v))}</td>`).join('')}</tr>`).join('');
    return `<table class="ce-sens-table"><thead>${head}</thead><tbody>${body}</tbody></table>`;
}
