import { api } from '../api.js';
import { esc, fmtDateTime, md } from '../util.js';
import { t } from '../i18n.js';
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
            <h2>${esc(t('view.community.h2.new_thread', { category: cat.name }))}</h2>
            <form id="thread-form">
                <input name="title" placeholder="title" data-i18n-placeholder="common.placeholder.title" required>
                <textarea name="body_md" placeholder="markdown body" data-i18n-placeholder="view.community.placeholder.body" required></textarea>
                <button data-i18n="view.community.btn.post" class="primary" type="submit">Post</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr><th data-i18n="view.community.th.title">Title</th><th data-i18n="view.community.th.posts">Posts</th><th data-i18n="view.community.th.views">Views</th><th data-i18n="view.community.th.last_post">Last post</th></tr></thead>
            <tbody>${threads.map(th => `
                <tr>
                    <td>${th.is_pinned ? '📌 ' : ''}<a href="#community/${cat.slug}/${th.slug}">${esc(th.title)}</a></td>
                    <td>${th.post_count}</td>
                    <td>${th.view_count}</td>
                    <td>${fmtDateTime(th.last_post_at)}</td>
                </tr>`).join('') || `<tr><td colspan="4" class="muted">${esc(t('view.community.empty.threads'))}</td></tr>`}
            </tbody>
        </table>

        <div class="chart-panel">
            <h2 data-i18n="view.community.h2.engagement_chart">Top threads — views vs posts</h2>
            <div id="comm-chart" style="width:100%;height:240px"></div>
        </div>
    `;
    renderEngagementChart(threads);
    mount.querySelector('#thread-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.forumCreateThread(cat.id, fd.get('title'), fd.get('body_md'));
        if (!viewIsCurrent(tok)) return;
        renderCommunity(mount, _state, cat.slug);
    });
}

function renderEngagementChart(threads) {
    const el = document.getElementById('comm-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const top = (threads || [])
        .filter(th => Number.isFinite(Number(th.view_count)) || Number.isFinite(Number(th.post_count)))
        .sort((a, b) => Number(b.view_count || 0) - Number(a.view_count || 0))
        .slice(0, 20);
    if (top.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.community.empty_chart">${esc(t('view.community.empty_chart'))}</div>`;
        return;
    }
    const labels = top.map(th => th.title.slice(0, 20));
    const views = top.map(th => Number(th.view_count || 0));
    const posts = top.map(th => Number(th.post_count || 0));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.community.chart.thread_idx') },
            { label: t('view.community.chart.views'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.community.chart.posts'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 6, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, views, posts], el);
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
                    <textarea name="body_md" placeholder="markdown reply" data-i18n-placeholder="view.community.placeholder.reply" required></textarea>
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
