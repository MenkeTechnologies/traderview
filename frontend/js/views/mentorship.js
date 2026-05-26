import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';

export async function renderMentorship(mount) {
    const [mentors, mentees] = await Promise.all([api.mentors(), api.mentees()]);
    mount.innerHTML = `
        <h1 class="view-title">// MENTORSHIP</h1>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2>Your mentors (people watching you)</h2>
                ${listTable(mentors, 'mentor_id', false)}
            </div>
            <div class="chart-panel">
                <h2>Your mentees (people you watch)</h2>
                ${listTable(mentees, 'mentee_id', true)}
            </div>
            <div class="chart-panel">
                <h2>Invite a mentor (paste their user ID)</h2>
                <form id="mentor-form" class="inline-form">
                    <input name="mentor_id" placeholder="mentor user UUID" required>
                    <select name="scope">
                        <option value="read">read-only</option>
                        <option value="comment">read + comment</option>
                    </select>
                    <button class="primary" type="submit">Invite</button>
                </form>
            </div>
        </div>
    `;

    document.getElementById('mentor-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.mentorshipRequest(fd.get('mentor_id'), fd.get('scope'));
        renderMentorship(mount);
    });

    document.querySelectorAll('[data-accept]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.acceptMentorship(b.dataset.accept);
            renderMentorship(mount);
        }));
    document.querySelectorAll('[data-revoke]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.revokeMentorship(b.dataset.revoke);
            renderMentorship(mount);
        }));
}

function listTable(rows, idCol, isMentor) {
    if (!rows.length) return '<p class="muted">None.</p>';
    return `<table class="trades"><thead><tr>
        <th>UUID</th><th>Status</th><th>Scope</th><th>Created</th><th>Accepted</th><th></th>
    </tr></thead><tbody>${rows.map(r => `
        <tr><td class="muted small">${esc(r[idCol])}</td>
        <td>${r.status}</td>
        <td>${r.scope}</td>
        <td>${fmtDateTime(r.created_at)}</td>
        <td>${r.accepted_at ? fmtDateTime(r.accepted_at) : '—'}</td>
        <td>
            ${isMentor && r.status === 'pending' ? `<button class="link" data-accept="${r.id}">accept</button>` : ''}
            <button class="link" data-revoke="${r.id}">revoke</button>
        </td></tr>
    `).join('')}</tbody></table>`;
}
