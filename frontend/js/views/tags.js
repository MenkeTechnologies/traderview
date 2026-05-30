import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderTags(mount) {
    const tok = currentViewToken();
    const tags = await api.tags();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.tags.h1.tags" class="view-title">// TAGS</h1>
        <div class="chart-panel">
            <h2 data-i18n="view.tags.h2.add_tag">Add tag</h2>
            <form id="tag-form" class="inline-form">
                <input name="name" placeholder="tag name" data-i18n-placeholder="view.tags.placeholder.name" required>
                <input type="color" name="color" value="#00e5ff">
                <button data-i18n="view.tags.btn.create" class="primary" type="submit">Create</button>
            </form>
        </div>
        <div class="tag-list">${tags.map(t => `
            <span class="tag-chip"
                  data-context-scope="tag-chip"
                  data-id="${esc(t.id)}"
                  data-name="${esc(t.name)}"
                  style="border-color:${esc(t.color)}">
                ${esc(t.name)}
                <button class="link" data-del="${t.id}" data-i18n-aria-label="common.aria.remove" aria-label="Remove">×</button>
            </span>
        `).join('') || '<p data-i18n="view.tags.hint.no_tags_yet" class="muted">No tags yet.</p>'}</div>
    `;
    mount.querySelector('#tag-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.createTag(fd.get('name'), fd.get('color'));
        if (!viewIsCurrent(tok)) return;
        renderTags(mount);
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteTag(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            renderTags(mount);
        }));
}
