import { api } from '../api.js';
import { esc, fmtDateTime, md } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderCommunity(mount, _state, catSlug) {
    const tok = currentViewToken();
    const cats = await api.forumCategories();
    if (!viewIsCurrent(tok)) return;
    if (!cats.length) { mount.innerHTML = '<p data-i18n="view.community.hint.no_forum_categories" class="boot">No forum categories.</p>'; return; }
    if (!catSlug) catSlug = cats[0].slug;
    const cat = cats.find(c => c.slug === catSlug) || cats[0];
    const threads = await api.forumThreadsIn(cat.slug);
    if (!viewIsCurrent(tok)) return;

    mount.innerHTML = `
        <h1 data-i18n="view.community.h1.community" class="view-title">// COMMUNITY</h1>
        <div class="forum-cats">${cats.map(c =>
            `<a class="forum-cat ${c.id === cat.id ? 'active' : ''}" href="#community/${c.slug}">${esc(c.name)}</a>`
        ).join('')}</div>

        <div class="chart-panel">
            <h2>New thread in ${esc(cat.name)}</h2>
            <form id="thread-form">
                <input name="title" placeholder="title" required>
                <textarea name="body_md" placeholder="markdown body" required></textarea>
                <button data-i18n="view.community.btn.post" class="primary" type="submit">Post</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr><th data-i18n="view.community.th.title">Title</th><th data-i18n="view.community.th.posts">Posts</th><th data-i18n="view.community.th.views">Views</th><th data-i18n="view.community.th.last_post">Last post</th></tr></thead>
            <tbody>${threads.map(t => `
                <tr>
                    <td>${t.is_pinned ? '📌 ' : ''}<a href="#community/${cat.slug}/${t.slug}">${esc(t.title)}</a></td>
                    <td>${t.post_count}</td>
                    <td>${t.view_count}</td>
                    <td>${fmtDateTime(t.last_post_at)}</td>
                </tr>`).join('') || '<tr><td colspan="4" class="muted">No threads yet.</td></tr>'}
            </tbody>
        </table>
    `;
    mount.querySelector('#thread-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.forumCreateThread(cat.id, fd.get('title'), fd.get('body_md'));
        if (!viewIsCurrent(tok)) return;
        renderCommunity(mount, _state, cat.slug);
    });
}

export async function renderCommunityThread(mount, _state, catSlug, threadSlug) {
    const tok = currentViewToken();
    const thread = await api.forumThreadBySlug(catSlug, threadSlug);
    if (!viewIsCurrent(tok)) return;
    api.forumBumpView(thread.id).catch(() => {});
    const posts = await api.forumPosts(thread.id);
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title">// ${esc(thread.title)}
            <a class="link small" href="#community/${catSlug}">← back</a></h1>
        <div class="thread-posts">${posts.map(p => `
            <div class="forum-post ${p.is_op ? 'op' : ''}">
                <div class="meta">${fmtDateTime(p.created_at)}</div>
                <div class="body">${md(p.body_md)}</div>
            </div>
        `).join('')}</div>
        ${thread.is_locked ? '<p data-i18n="view.community.hint.thread_is_locked" class="muted">Thread is locked.</p>' : `
            <div class="chart-panel">
                <h2 data-i18n="view.community.h2.reply">Reply</h2>
                <form id="reply-form">
                    <textarea name="body_md" placeholder="markdown reply" required></textarea>
                    <button data-i18n="view.community.btn.post_2" class="primary" type="submit">Post</button>
                </form>
            </div>
        `}
    `;
    if (!thread.is_locked) {
        mount.querySelector('#reply-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const fd = new FormData(e.target);
            await api.forumCreatePost(thread.id, fd.get('body_md'));
            if (!viewIsCurrent(tok)) return;
            renderCommunityThread(mount, _state, catSlug, threadSlug);
        });
    }
}
