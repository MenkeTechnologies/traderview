// Backtest preset library — save, share via slug, fork community presets.
import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderBacktestPresets(mount, _state, slug = '') {
    if (slug) return renderPresetDetail(mount, slug);
    return renderBrowse(mount);
}

async function renderBrowse(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// BACKTEST PRESETS</h1>
        <p class="muted small">Save parameter combinations under a name; mark them public to
            get a shareable slug; fork others' public presets to your own library. Run a preset
            by feeding its <code>preset</code> JSON straight into the Backtest tab.</p>

        <div class="chart-panel">
            <h2>Save a new preset</h2>
            <form id="bp-form" class="inline-form">
                <input name="name" placeholder="name (unique per user)" required style="min-width:200px;">
                <input name="description" placeholder="optional description" style="min-width:240px;">
                <label><input name="is_public" type="checkbox"> public</label>
                <button class="primary" type="submit">Save</button>
            </form>
            <textarea id="bp-json" rows="8" placeholder='Preset JSON, e.g.
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
            <h2>My presets</h2>
            <div id="bp-mine"><div class="boot">loading…</div></div>
        </div>
        <div class="chart-panel">
            <h2>Public library (top by forks)</h2>
            <div id="bp-public"><div class="boot">loading…</div></div>
        </div>
    `;
    mount.querySelector('#bp-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        let preset;
        try { preset = JSON.parse(mount.querySelector('#bp-json').value); }
        catch (err) { alert('preset JSON invalid: ' + err.message); return; }
        try {
            await api.createBacktestPreset({
                name: fd.get('name').trim(),
                description: fd.get('description') || null,
                preset,
                is_public: !!fd.get('is_public'),
            });
            if (!viewIsCurrent(tok)) return;
            e.target.reset();
            const ta = mount.querySelector('#bp-json');
            if (ta) ta.value = '';
            await refresh(mount, tok);
        } catch (err) { alert(err.message); }
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
    if (!rows.length) { el.innerHTML = '<p class="muted small">No presets yet.</p>'; return; }
    el.innerHTML = table(rows, true);
    wireRowButtons(el, true, mount, tok);
}

function renderPublic(rows, mount, tok) {
    const el = mount.querySelector('#bp-public');
    if (!el) return;
    if (!rows.length) { el.innerHTML = '<p class="muted small">No public presets yet — be the first.</p>'; return; }
    el.innerHTML = table(rows, false);
    wireRowButtons(el, false, mount, tok);
}

function table(rows, mine) {
    return `<table class="trades">
        <thead><tr>
            <th>Name</th><th>Visibility</th><th>Slug</th>
            <th>Forks</th><th>Runs</th><th>Updated</th><th></th>
        </tr></thead>
        <tbody>
        ${rows.map(r => `<tr>
            <td><a href="#backtest-presets/${esc(r.slug)}">${esc(r.name)}</a>
                ${r.description ? `<div class="muted small">${esc(r.description)}</div>` : ''}
            </td>
            <td class="small ${r.is_public ? 'pos' : 'muted'}">${r.is_public ? 'public' : 'private'}</td>
            <td><code class="small">${esc(r.slug)}</code></td>
            <td>${r.fork_count}</td>
            <td>${r.run_count}</td>
            <td class="small">${new Date(r.updated_at).toLocaleDateString()}</td>
            <td>
                ${mine
                    ? `<button class="btn bp-del" data-id="${r.id}">Delete</button>`
                    : `<button class="btn bp-fork" data-slug="${esc(r.slug)}">Fork</button>`}
            </td>
        </tr>`).join('')}
        </tbody></table>`;
}

function wireRowButtons(scope, mine, mount, tok) {
    if (mine) {
        scope.querySelectorAll('.bp-del').forEach(b => {
            b.addEventListener('click', async () => {
                if (!confirm('Delete this preset?')) return;
                try { await api.deleteBacktestPreset(b.dataset.id); if (viewIsCurrent(tok)) await refresh(mount, tok); }
                catch (e) { alert(e.message); }
            });
        });
    } else {
        scope.querySelectorAll('.bp-fork').forEach(b => {
            b.addEventListener('click', async () => {
                try {
                    const forked = await api.forkBacktestPreset(b.dataset.slug);
                    alert(`Forked as "${forked.name}"`);
                    if (viewIsCurrent(tok)) await refresh(mount, tok);
                } catch (e) { alert(e.message); }
            });
        });
    }
}

async function renderPresetDetail(mount, slug) {
    const tok = currentViewToken();
    mount.innerHTML = `<h1 class="view-title">// PRESET — ${esc(slug)}</h1>
        <div class="boot">loading…</div>`;
    try {
        const r = await api.getBacktestPresetBySlug(slug);
        if (!viewIsCurrent(tok)) return;
        mount.innerHTML = `
            <h1 class="view-title">// PRESET — ${esc(r.name)}</h1>
            <div class="cards">
                <div class="card"><div class="label">Visibility</div>
                    <div class="value ${r.is_public ? 'pos' : 'muted'}">${r.is_public ? 'public' : 'private'}</div></div>
                <div class="card"><div class="label">Forks</div>
                    <div class="value">${r.fork_count}</div></div>
                <div class="card"><div class="label">Runs</div>
                    <div class="value">${r.run_count}</div></div>
                <div class="card"><div class="label">Slug</div>
                    <div class="value small"><code>${esc(r.slug)}</code></div></div>
            </div>
            ${r.description ? `<div class="chart-panel"><p>${esc(r.description)}</p></div>` : ''}
            <div class="chart-panel">
                <h2>Preset JSON</h2>
                <pre style="background:#070714;padding:8px;font-size:11px;overflow:auto;">${esc(JSON.stringify(r.preset, null, 2))}</pre>
                <button class="btn" id="bp-fork-btn">Fork to my library</button>
                <a class="btn" href="#backtest-presets" style="margin-left:6px;">Back to library</a>
            </div>
        `;
        const btn = mount.querySelector('#bp-fork-btn');
        if (btn) btn.addEventListener('click', async () => {
            try {
                const f = await api.forkBacktestPreset(slug);
                alert(`Forked as "${f.name}"`);
                window.location.hash = `backtest-presets/${f.slug}`;
            } catch (e) { alert(e.message); }
        });
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        mount.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
