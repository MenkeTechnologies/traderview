// Shared tabbed-calculator renderer. A view supplies a TOOLS map
// ({ key: { label, call, fields, render } }) and this draws the tab
// row + form + result panel and wires submit → call → render. Used by
// valuation_tools.js and strategy_tools.js — extend the TOOLS map, not
// this file.
//
// Field flags: text (uppercased string input), int (parseInt),
// optional (empty input → null in the payload).

import { esc } from '../util.js';
import { applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export function renderToolTabs(mount, { titleKey, title, hintKey, tools, defaultKey }) {
    const tok = currentViewToken();
    const many = Object.keys(tools).length > 8;
    mount.innerHTML = `
        <h1 data-i18n="${titleKey}" class="view-title">${esc(title)}</h1>
        ${many ? '<div class="gs-filter-row"><input type="text" class="tt-filter" placeholder="filter tools…"></div>' : ''}
        <div class="gs-filter-row vt-tabs">
            ${Object.entries(tools).map(([k, v]) => `
                <button class="btn btn-secondary gs-filter vt-tab" data-key="${k}">${esc(v.label)}</button>
            `).join('')}
        </div>
        <div class="chart-panel"><div class="tt-body"></div></div>
        <div class="chart-panel"><div class="tt-result muted" data-i18n="${hintKey}">Pick a tool, fill the form, hit Compute.</div></div>
    `;
    try { applyUiI18n(mount); } catch (_) {}

    const filterBox = mount.querySelector('.tt-filter');
    if (filterBox) {
        filterBox.addEventListener('input', () => {
            const q = filterBox.value.trim().toLowerCase();
            mount.querySelectorAll('.vt-tab').forEach(b => {
                b.classList.toggle('tt-hidden', q !== '' && !b.textContent.toLowerCase().includes(q));
            });
        });
    }

    const body = mount.querySelector('.tt-body');
    const out = mount.querySelector('.tt-result');

    const show = (key) => {
        const tool = tools[key];
        mount.querySelectorAll('.vt-tab').forEach(b =>
            b.classList.toggle('active', b.dataset.key === key));
        body.innerHTML = `
            <form class="dcf-form" data-tool="${key}">
                ${tool.fields.map(f => `
                    <label class="dcf-field">
                        <span class="dcf-label">${esc(f.label)}</span>
                        <input name="${f.key}" type="${f.text ? 'text' : 'number'}"
                               ${f.text ? 'style="text-transform:uppercase"' : 'step="any"'}
                               value="${f.def}">
                    </label>`).join('')}
                <button type="submit" class="primary">Compute</button>
            </form>`;
        body.querySelector('form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const fd = new FormData(e.target);
            const payload = {};
            for (const f of tool.fields) {
                const raw = fd.get(f.key);
                if (f.optional && raw === '') { payload[f.key] = null; continue; }
                if (f.text) { payload[f.key] = String(raw).trim().toUpperCase(); continue; }
                payload[f.key] = f.int ? (parseInt(raw, 10) || 0) : (Number(raw) || 0);
            }
            out.textContent = '…';
            try {
                const r = await tool.call(payload);
                if (!viewIsCurrent(tok)) return;
                out.innerHTML = tool.render(r);
            } catch (err) {
                out.innerHTML = `<span class="neg">${esc(err.message || String(err))}</span>`;
            }
        });
    };
    mount.querySelectorAll('.vt-tab').forEach(b =>
        b.addEventListener('click', () => show(b.dataset.key)));
    show(defaultKey || Object.keys(tools)[0]);
}
