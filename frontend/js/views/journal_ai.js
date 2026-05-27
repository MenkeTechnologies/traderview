// AI journal analysis — LLM settings panel + per-trade Analyze button.
// Two views in one module:
//   renderAiSettings(mount) — provider/model/key/etc. form
//   renderAiAnalyze(mount, tradeId) — fetches cached, button to (re-)analyze

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderAiSettings(mount) {
    const tok = currentViewToken();
    const cfg = await api.getAiSettings().catch(() => ({}));
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title">// AI / LLM SETTINGS</h1>
        <p class="muted small">Configure a provider for AI trade-journal analysis.
            Anthropic / OpenAI require an API key; Ollama runs locally with no key.
            The stored key is redacted on read — leave the field blank to keep the
            current key when saving other fields.</p>

        <form id="ai-form" class="inline-form">
            <label>Provider
                <select name="provider">
                    <option value="">(none)</option>
                    <option value="anthropic" ${cfg.provider === 'anthropic' ? 'selected' : ''}>Anthropic</option>
                    <option value="openai"    ${cfg.provider === 'openai'    ? 'selected' : ''}>OpenAI</option>
                    <option value="ollama"    ${cfg.provider === 'ollama'    ? 'selected' : ''}>Ollama (local)</option>
                </select>
            </label>
            <label>Model
                <input name="model" placeholder="claude-haiku-4-5-20251001 / gpt-4o-mini / llama3"
                       value="${esc(cfg.model || '')}" style="min-width:280px;">
            </label>
            <label>Endpoint (override)
                <input name="endpoint" placeholder="default per provider"
                       value="${esc(cfg.endpoint || '')}" style="min-width:240px;">
            </label>
            <label>API key
                <input name="api_key" type="password"
                       placeholder="${cfg.api_key ? '*** (saved, leave blank to keep)' : 'paste key'}"
                       autocomplete="off">
            </label>
            <label>Max tokens <input name="max_tokens" type="number" min="100" max="4000"
                                    value="${cfg.max_tokens ?? 800}" style="width:90px;"></label>
            <label>Temp <input name="temperature" type="number" min="0" max="2" step="0.05"
                              value="${cfg.temperature ?? 0.2}" style="width:80px;"></label>
            <button class="primary" type="submit">Save</button>
            <span id="ai-save-status" class="muted small"></span>
        </form>

        <p class="muted small">Defaults: Anthropic → <code>https://api.anthropic.com</code>,
            OpenAI → <code>https://api.openai.com</code>, Ollama → <code>http://localhost:11434</code>.
            The prompt asks the model to return strict JSON with summary / mistakes /
            risk_gaps / suggestions / rule_changes arrays.</p>
    `;
    mount.querySelector('#ai-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            provider: fd.get('provider') || null,
            model: fd.get('model') || null,
            endpoint: fd.get('endpoint') || null,
            api_key: fd.get('api_key') || null,
            max_tokens: Number(fd.get('max_tokens')) || null,
            temperature: Number(fd.get('temperature')) || null,
        };
        const status = mount.querySelector('#ai-save-status');
        if (status) status.textContent = 'saving…';
        try {
            await api.setAiSettings(body);
            if (!viewIsCurrent(tok)) return;
            const s2 = mount.querySelector('#ai-save-status');
            if (s2) {
                s2.textContent = 'saved';
                setTimeout(() => {
                    if (!viewIsCurrent(tok)) return;
                    const s3 = mount.querySelector('#ai-save-status');
                    if (s3) s3.textContent = '';
                }, 2000);
            }
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const s2 = mount.querySelector('#ai-save-status');
            if (s2) s2.textContent = 'error: ' + err.message;
        }
    });
}

/// Render the AI-analyze panel for a single trade. Caller passes a mount
/// (typically a container appended inside the trade detail view).
export async function renderAiAnalyze(mount, tradeId) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <div class="chart-panel">
            <h2>AI analysis</h2>
            <div id="ai-status" class="muted small">checking cache…</div>
            <div id="ai-body"></div>
            <button class="btn" id="ai-run">Run analysis</button>
        </div>
    `;
    const status = mount.querySelector('#ai-status');
    const body = mount.querySelector('#ai-body');

    async function loadCached() {
        try {
            const cached = await api.getAiCached(tradeId);
            if (!viewIsCurrent(tok)) return;
            if (cached) {
                if (status) status.textContent = `cached ${new Date(cached.created_at).toLocaleString()} · ${cached.provider}/${cached.model}` +
                    (cached.prompt_tokens ? ` · ${cached.prompt_tokens}+${cached.response_tokens || 0} tok` : '');
                if (body) body.innerHTML = renderFindings(cached.findings);
                const runBtn = mount.querySelector('#ai-run');
                if (runBtn) runBtn.textContent = 'Re-analyze';
            } else {
                if (status) status.textContent = 'no cached analysis for this trade';
            }
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            if (status) status.textContent = 'cache lookup failed: ' + e.message;
        }
    }
    await loadCached();
    if (!viewIsCurrent(tok)) return;

    const runBtn = mount.querySelector('#ai-run');
    if (!runBtn) return;
    runBtn.addEventListener('click', async () => {
        const btn = mount.querySelector('#ai-run');
        if (!btn) return;
        btn.disabled = true;
        if (status) status.textContent = 'analyzing… (LLM call may take 5-30s)';
        try {
            const r = await api.runAiAnalysis(tradeId);
            if (!viewIsCurrent(tok)) return;
            if (status) status.textContent = `done · ${r.provider}/${r.model}` +
                (r.prompt_tokens ? ` · ${r.prompt_tokens}+${r.response_tokens || 0} tok` : '');
            if (body) body.innerHTML = renderFindings(r.findings);
            btn.textContent = 'Re-analyze';
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            if (status) status.textContent = 'error: ' + e.message;
        } finally {
            if (viewIsCurrent(tok)) btn.disabled = false;
        }
    });
}

function renderFindings(f) {
    const list = (title, arr, cls = '') => {
        if (!arr || !arr.length) return '';
        return `<h3 style="margin-top:12px;">${esc(title)}</h3>
            <ul class="${cls}">${arr.map(s => `<li>${esc(s)}</li>`).join('')}</ul>`;
    };
    return `
        <p style="font-size:14px;"><strong>${esc(f.summary || '')}</strong></p>
        ${list('Mistakes', f.mistakes, 'neg')}
        ${list('Risk gaps', f.risk_gaps, 'neg')}
        ${list('Suggestions', f.suggestions)}
        ${list('Rule changes', f.rule_changes, 'pos')}
    `;
}
