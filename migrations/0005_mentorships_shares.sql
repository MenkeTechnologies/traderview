-- 0005 — mentorships + public trade shares + comments
-- TraderVue parity: mentor/mentee read-only journal sharing, public trade pages, comment threads.

CREATE TYPE mentorship_status_t AS ENUM ('pending', 'active', 'revoked');

-- Mentor/mentee relationships. A user can have many mentors and many mentees.
CREATE TABLE mentorships (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mentor_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mentee_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status          mentorship_status_t NOT NULL DEFAULT 'pending',
    scope           TEXT NOT NULL DEFAULT 'read',             -- read | comment (mentor may leave notes)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    accepted_at     TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    UNIQUE (mentor_id, mentee_id),
    CHECK (mentor_id <> mentee_id)
);
CREATE INDEX mentorships_mentor_idx ON mentorships(mentor_id, status);
CREATE INDEX mentorships_mentee_idx ON mentorships(mentee_id, status);

-- A mentor's read-only view of a mentee's trades — denormalized for query speed.
-- (no separate ACL — `mentorships` IS the ACL; routes JOIN against it.)

-- ---------------------------------------------------------------------------
-- Public trade shares — a sharable slug renders one trade publicly.
-- ---------------------------------------------------------------------------
CREATE TABLE trade_shares (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id        UUID NOT NULL REFERENCES trades(id) ON DELETE CASCADE,
    owner_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    slug            TEXT NOT NULL UNIQUE,                     -- url-safe short id, e.g. 'k3p8q2'
    is_public       BOOLEAN NOT NULL DEFAULT TRUE,            -- false = link-only / unlisted
    show_notes      BOOLEAN NOT NULL DEFAULT TRUE,
    show_screenshots BOOLEAN NOT NULL DEFAULT TRUE,
    view_count      BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ
);
CREATE INDEX trade_shares_owner_idx ON trade_shares(owner_id, created_at DESC);
CREATE INDEX trade_shares_public_idx ON trade_shares(is_public, created_at DESC) WHERE is_public = TRUE;

-- Comment threads on shared trades.
CREATE TABLE comments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    share_id        UUID NOT NULL REFERENCES trade_shares(id) ON DELETE CASCADE,
    author_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES comments(id) ON DELETE CASCADE,   -- threaded
    body_md         TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX comments_share_idx ON comments(share_id, created_at);
CREATE INDEX comments_author_idx ON comments(author_id);
CREATE INDEX comments_parent_idx ON comments(parent_id);
