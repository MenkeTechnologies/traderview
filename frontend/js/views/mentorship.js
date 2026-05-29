import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderMentorship(mount) {
    const tok = currentViewToken();
    const [mentors, mentees] = await Promise.all([api.mentors(), api.mentees()]);
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.mentorship.h1.mentorship" class="view-title">// MENTORSHIP</h1>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.mentorship.h2.your_mentors_people_watching_you">Your mentors (people watching you)</h2>
                ${listTable(mentors, 'mentor_id', false)}
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.mentorship.h2.your_mentees_people_you_watch">Your mentees (people you watch)</h2>
                ${listTable(mentees, 'mentee_id', true)}
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.mentorship.h2.invite_a_mentor_paste_their_user_id">Invite a mentor (paste their user ID)</h2>
                <form id="mentor-form" class="inline-form">
                    <input name="mentor_id" placeholder="mentor user UUID" data-i18n-placeholder="view.mentorship.placeholder.mentor" required>
                    <select name="scope">
                        <option data-i18n="view.mentorship.opt.read_only" value="read">read-only</option>
                        <option data-i18n="view.mentorship.opt.read_comment" value="comment">read + comment</option>
                    </select>
                    <button data-i18n="view.mentorship.btn.invite" class="primary" type="submit">Invite</button>
                </form>
            </div>
        </div>
    `;

    mount.querySelector('#mentor-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.mentorshipRequest(fd.get('mentor_id'), fd.get('scope'));
        if (!viewIsCurrent(tok)) return;
        renderMentorship(mount);
    });

    mount.querySelectorAll('[data-accept]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.acceptMentorship(b.dataset.accept);
            if (!viewIsCurrent(tok)) return;
            renderMentorship(mount);
        }));
    mount.querySelectorAll('[data-revoke]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.revokeMentorship(b.dataset.revoke);
            if (!viewIsCurrent(tok)) return;
            renderMentorship(mount);
        }));
}

function listTable(rows, idCol, isMentor) {
    if (!rows.length) return '<p data-i18n="view.mentorship.hint.none" class="muted">None.</p>';
    return `<table class="trades"><thead><tr>
        <th data-i18n="view.mentorship.th.uuid">UUID</th><th data-i18n="view.mentorship.th.status">Status</th><th data-i18n="view.mentorship.th.scope">Scope</th><th data-i18n="view.mentorship.th.created">Created</th><th data-i18n="view.mentorship.th.accepted">Accepted</th><th></th>
    </tr></thead><tbody>${rows.map(r => `
        <tr><td class="muted small">${esc(r[idCol])}</td>
        <td>${r.status}</td>
        <td>${r.scope}</td>
        <td>${fmtDateTime(r.created_at)}</td>
        <td>${r.accepted_at ? fmtDateTime(r.accepted_at) : '—'}</td>
        <td>
            ${isMentor && r.status === 'pending' ? `<button data-i18n="view.mentorship.btn.accept" class="link" data-accept="${r.id}">accept</button>` : ''}
            <button data-i18n="view.mentorship.btn.revoke" class="link" data-revoke="${r.id}">revoke</button>
        </td></tr>
    `).join('')}</tbody></table>`;
}
