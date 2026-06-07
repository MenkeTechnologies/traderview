// Backtest preset library — save, share via slug, fork community presets.
import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';

export async function renderBacktestPresets(mount, _state, slug = '') {
    if (slug) return renderPresetDetail(mount, slug);
    return renderBrowse(mount);
}

async function renderBrowse(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.backtest_presets.h1.backtest_presets" class="view-title">// BACKTEST PRESETS</h1>
        <p class="muted small" data-i18n="view.backtest_presets.hint.intro">Save parameter combinations under a name; mark them public to get a shareable slug; fork others' public presets to your own library. Run a preset by feeding its preset JSON straight into the Backtest tab.</p>

        <div class="chart-panel">
            <h2 data-i18n="view.backtest_presets.h2.save_a_new_preset">Save a new preset</h2>
            <form id="bp-form" class="inline-form">
                <input name="name" placeholder="name (unique per user)" data-i18n-placeholder="view.backtest_presets.placeholder.name"
                       data-tip="view.backtest_presets.tip.name" data-shortcut="backtest_presets_focus_name"
                       required style="min-width:200px;">
                <input name="description" placeholder="optional description" data-i18n-placeholder="view.backtest_presets.placeholder.description"
                       data-tip="view.backtest_presets.tip.description" style="min-width:240px;">
                <label><input name="is_public" type="checkbox" data-tip="view.backtest_presets.tip.public"> <span data-i18n="view.backtest_presets.label.public">public</span></label>
                <button data-i18n="view.backtest_presets.btn.save" data-tip="view.backtest_presets.tip.save" class="primary" type="submit">Save</button>
            </form>
            <textarea id="bp-json" rows="8"
                data-i18n-placeholder="view.backtest_presets.placeholder.preset"
                data-tip="view.backtest_presets.tip.preset_json"
                placeholder='Preset JSON, e.g.
{
  "symbol": "SPY",
  "preset": { "sma_cross": { "fast": 20, "slow": 50 } },
  "days": 730,
  "initial_capital": 10000,
  "fee_per_trade": 1
}'
                style="width:100%;font-family:'Share Tech Mono',monospace;font-size:11px;background:#070714;color:#cfd2e8;border:1px solid var(--border);padding:8px;margin-top:8px;"></textarea>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.backtest_presets.h2.my_presets">My presets</h2>
            <div id="bp-mine"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.backtest_presets.h2.public_library_top_by_forks">Public library (top by forks)</h2>
            <div id="bp-public"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.backtest_presets.h2.public_chart">Public presets by forks (top 20)</h2>
            <div id="bp-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.backtest_presets.h2.age_chart">Public preset age (days since last update)</h2>
            <div id="bp-age-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.backtest_presets.hint.age_chart" class="muted small">Days since each top-20 public preset's last <code>updated_at</code>. Reveals which popular presets are actively maintained vs stale clones. Orthogonal to the forks/runs popularity chart above.</p>
        </div>
    `;
    mount.querySelector('#bp-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        let preset;
        try { preset = JSON.parse(mount.querySelector('#bp-json').value); }
        catch (err) { showToast(t('view.backtest_presets.alert.json_invalid', { err: err.message }), { level: 'error' }); return; }
        const name = String(fd.get('name') || '').trim();
        try {
            await api.createBacktestPreset({
                name,
                description: fd.get('description') || null,
                preset,
                is_public: !!fd.get('is_public'),
            });
            if (!viewIsCurrent(tok)) return;
            e.target.reset();
            const ta = mount.querySelector('#bp-json');
            if (ta) ta.value = '';
            showToast(t('view.backtest_presets.toast.saved', { name }), { level: 'success' });
            await refresh(mount, tok);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    try {
        const [mine, pub_] = await Promise.all([
            api.listMyBacktestPresets(),
            api.listPublicBacktestPresets(),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderMine(mine, mount, tok);
        renderPublic(pub_, mount, tok);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#bp-mine');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderMine(rows, mount, tok) {
    const el = mount.querySelector('#bp-mine');
    if (!el) return;
    if (!rows.length) { el.innerHTML = '<p data-i18n="view.backtest_presets.hint.no_presets_yet" class="muted small">No presets yet.</p>'; return; }
    el.innerHTML = table(rows, true);
    wireRowButtons(el, true, mount, tok);
}

function renderPublic(rows, mount, tok) {
    const el = mount.querySelector('#bp-public');
    if (!el) return;
    if (!rows.length) { el.innerHTML = '<p data-i18n="view.backtest_presets.hint.no_public_presets_yet_be_the_first" class="muted small">No public presets yet — be the first.</p>'; return; }
    el.innerHTML = table(rows, false);
    wireRowButtons(el, false, mount, tok);
    renderForksChart(rows);
    renderAgeChart(rows);
}

function renderForksChart(rows) {
    const el = document.getElementById('bp-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const top = (rows || [])
        .filter(r => Number.isFinite(Number(r.fork_count)) || Number.isFinite(Number(r.forks)))
        .map(r => ({
            name: r.name,
            forks: Number(r.fork_count != null ? r.fork_count : (r.forks || 0)),
            runs: Number(r.run_count != null ? r.run_count : (r.runs || 0)),
        }))
        .sort((a, b) => b.forks - a.forks)
        .slice(0, 20);
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.backtest_presets.empty_chart">${esc(t('view.backtest_presets.empty_chart'))}</div>`;
        return;
    }
    const labels = top.map(r => r.name);
    const forks = top.map(r => r.forks);
    const runs = top.map(r => r.runs);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.backtest_presets.chart.preset_idx') },
            { label: t('view.backtest_presets.chart.forks'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.backtest_presets.chart.runs'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 6, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, forks, runs], el);
}

function renderAgeChart(rows) {
    const el = document.getElementById('bp-age-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const now = Date.now();
    const top = (rows || [])
        .filter(r => r && r.updated_at)
        .map(r => ({
            name: r.name || '?',
            age: (now - new Date(r.updated_at).getTime()) / (1000 * 60 * 60 * 24),
            forks: Number(r.fork_count != null ? r.fork_count : (r.forks || 0)),
        }))
        .filter(r => Number.isFinite(r.age))
        .sort((a, b) => b.forks - a.forks)
        .slice(0, 20);
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.backtest_presets.empty_age_chart">${esc(t('view.backtest_presets.empty_age_chart'))}</div>`;
        return;
    }
    const labels = top.map(r => r.name);
    const ys = top.map(r => r.age);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.backtest_presets.chart.preset_idx') },
            { label: t('view.backtest_presets.chart.age_days'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50,
              values: (_u, splits) => splits.map(v => v.toFixed(0) + 'd') },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function table(rows, mine) {
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.backtest_presets.th.name">Name</th><th data-i18n="view.backtest_presets.th.visibility">Visibility</th><th data-i18n="view.backtest_presets.th.slug">Slug</th>
            <th data-i18n="view.backtest_presets.th.forks">Forks</th><th data-i18n="view.backtest_presets.th.runs">Runs</th><th data-i18n="view.backtest_presets.th.updated">Updated</th><th></th>
        </tr></thead>
        <tbody>
        ${rows.map(r => `<tr data-context-scope="backtest-preset-row"
                              data-id="${esc(r.id)}"
                              data-slug="${esc(r.slug)}"
                              data-name="${esc(r.name)}"
                              data-mine="${mine ? 'true' : 'false'}">
            <td><a href="#backtest-presets/${esc(r.slug)}">${esc(r.name)}</a>
                ${r.description ? `<div class="muted small">${esc(r.description)}</div>` : ''}
            </td>
            <td class="small ${r.is_public ? 'pos' : 'muted'}">${r.is_public ? t('common.status.public') : t('common.status.private')}</td>
            <td><code class="small">${esc(r.slug)}</code></td>
            <td>${r.fork_count}</td>
            <td>${r.run_count}</td>
            <td class="small">${new Date(r.updated_at).toLocaleDateString()}</td>
            <td>
                ${mine
                    ? `<button data-i18n="view.backtest_presets.btn.delete" data-tip="view.backtest_presets.tip.delete_row" class="btn bp-del" data-id="${r.id}">Delete</button>`
                    : `<button data-i18n="view.backtest_presets.btn.fork" data-tip="view.backtest_presets.tip.fork_row" class="btn bp-fork" data-slug="${esc(r.slug)}">Fork</button>`}
            </td>
        </tr>`).join('')}
        </tbody></table>`;
}

function wireRowButtons(scope, mine, mount, tok) {
    if (mine) {
        scope.querySelectorAll('.bp-del').forEach(b => {
            b.addEventListener('click', async () => {
                if (!await tConfirm('view.backtest_presets.confirm.delete', {}, { level: 'danger' })) return;
                try {
                    await api.deleteBacktestPreset(b.dataset.id);
                    showToast(t('view.backtest_presets.toast.deleted'), { level: 'success' });
                    if (viewIsCurrent(tok)) await refresh(mount, tok);
                } catch (e) { showToast(t('common.error', { err: e.message }), { level: 'error' }); }
            });
        });
    } else {
        scope.querySelectorAll('.bp-fork').forEach(b => {
            b.addEventListener('click', async () => {
                try {
                    const forked = await api.forkBacktestPreset(b.dataset.slug);
                    showToast(t('view.backtest_presets.alert.forked', { name: forked.name }), { level: 'success' });
                    if (viewIsCurrent(tok)) await refresh(mount, tok);
                } catch (e) { showToast(t('common.error', { err: e.message }), { level: 'error' }); }
            });
        });
    }
}

async function renderPresetDetail(mount, slug) {
    const tok = currentViewToken();
    mount.innerHTML = `<h1 class="view-title">// PRESET — ${esc(slug)}</h1>
        <div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div>`;
    try {
        const r = await api.getBacktestPresetBySlug(slug);
        if (!viewIsCurrent(tok)) return;
        mount.innerHTML = `
            <h1 class="view-title">// PRESET — ${esc(r.name)}</h1>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.backtest_presets.card.visibility">Visibility</div>
                    <div class="value ${r.is_public ? 'pos' : 'muted'}">${r.is_public ? t('common.status.public') : t('common.status.private')}</div></div>
                <div class="card"><div class="label" data-i18n="view.backtest_presets.card.forks">Forks</div>
                    <div class="value">${r.fork_count}</div></div>
                <div class="card"><div class="label" data-i18n="view.backtest_presets.card.runs">Runs</div>
                    <div class="value">${r.run_count}</div></div>
                <div class="card"><div class="label" data-i18n="view.backtest_presets.card.slug">Slug</div>
                    <div class="value small"><code>${esc(r.slug)}</code></div></div>
            </div>
            ${r.description ? `<div class="chart-panel"><p>${esc(r.description)}</p></div>` : ''}
            <div class="chart-panel">
                <h2 data-i18n="view.backtest_presets.h2.preset_json">Preset JSON</h2>
                <pre style="background:#070714;padding:8px;font-size:11px;overflow:auto;">${esc(JSON.stringify(r.preset, null, 2))}</pre>
                <button data-i18n="view.backtest_presets.btn.fork_to_my_library" class="btn" id="bp-fork-btn">Fork to my library</button>
                <a class="btn" href="#backtest-presets" data-i18n="view.backtest_presets.link.back_to_library" style="margin-left:6px;">Back to library</a>
            </div>
        `;
        const btn = mount.querySelector('#bp-fork-btn');
        if (btn) {
            btn.setAttribute('data-tip', 'view.backtest_presets.tip.fork_to_lib');
            btn.addEventListener('click', async () => {
                try {
                    const f = await api.forkBacktestPreset(slug);
                    showToast(t('view.backtest_presets.alert.forked', { name: f.name }), { level: 'success' });
                    window.location.hash = `backtest-presets/${f.slug}`;
                } catch (e) { showToast(t('common.error', { err: e.message }), { level: 'error' }); }
            });
        }
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        mount.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
