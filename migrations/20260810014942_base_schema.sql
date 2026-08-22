-- Game questions, sections, content etc. Each game has one template from which
-- the questions are drawn.
CREATE TABLE templates (
    id            INTEGER PRIMARY KEY,
    name          TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    text          TEXT NOT NULL
) STRICT;

-- Game instances
CREATE TABLE games (
    id            INTEGER PRIMARY KEY,
    template_id   INTEGER NOT NULL REFERENCES templates(id),
    join_code     BLOB NOT NULL,
    created_by    INTEGER NOT NULL REFERENCES admins(id),
    created_at    INTEGER NOT NULL
) STRICT;

-- Section titles instantiated from templates
CREATE TABLE sections (
    id            INTEGER PRIMARY KEY,
    game_id       INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    template_id   INTEGER NOT NULL REFERENCES templates(id),
    title         TEXT NOT NULL
) STRICT;

-- Questions instantiated from templates
CREATE TABLE questions (
    id            INTEGER PRIMARY KEY,
    game_id       INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    section       INTEGER NOT NULL REFERENCES sections(id),
    position      INTEGER NOT NULL,  -- Position of the question within the section
    text          TEXT NOT NULL,
    UNIQUE (game_id, section, position)
) STRICT;

-- Each team's answers
CREATE TABLE answers (
    id            INTEGER PRIMARY KEY,
    game_id       INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    team_id       INTEGER NOT NULL REFERENCES teams(id),
    question_id   INTEGER NOT NULL REFERENCES questions(id),
    answer        TEXT,  -- NULL answer means that the question hasn't been answered yet.
    is_correct    INTEGER,
    UNIQUE (team_id, question_id)
) STRICT;

-- Information about the administrators, who can create templates and run game instances.
CREATE TABLE admins (
    id            INTEGER PRIMARY KEY,
    username      TEXT NOT NULL,
    pw_hash       TEXT NOT NULL,  -- Salted hash in PHC format
    UNIQUE (username)
) STRICT;

-- Information about players. Each player is ephemeral and per-game (no
-- persistence across games for now).
CREATE TABLE players (
    id            INTEGER PRIMARY KEY,
    game_id       INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    team_id       INTEGER REFERENCES teams(id) ON DELETE SET NULL,
    nickname      TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    UNIQUE (id, game_id)
) STRICT;

-- Information about the teams
CREATE TABLE teams (
    id            INTEGER PRIMARY KEY,
    game_id       INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    join_code     BLOB NOT NULL,
    team_name     TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    UNIQUE(game_id, team_name)
) STRICT;

-- Ephemeral player sessions. Each session represents a player in a particular
-- game instance.
CREATE TABLE sessions (
    token_hash    BLOB PRIMARY KEY, -- SHA-256 hash of session token
    game_id       INTEGER REFERENCES games(id) ON DELETE CASCADE,
    admin_id      INTEGER REFERENCES admins(id),  -- null for player sessions
    player_id     INTEGER REFERENCES players(id), -- null for admin sessions
    created_at    INTEGER NOT NULL DEFAULT (unixepoch('now')),
    expires_at    INTEGER NOT NULL DEFAULT (unixepoch('now', '+24 hours')),

    FOREIGN KEY (player_id, game_id)
        REFERENCES players(id, game_id),

    CHECK (
        (admin_id IS NOT NULL AND player_id IS NULL AND game_id IS NULL)
        OR
        (admin_id IS NULL AND player_id IS NOT NULL AND game_id IS NOT NULL)
    )
) STRICT;

-- Game state to track progression of the game.
CREATE TABLE game_state (
    game_id             INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    phase               TEXT NOT NULL,
    current_question_id INTEGER REFERENCES questions(id),
    phase_entered_at    INTEGER NOT NULL,
    section_deadline    INTEGER,  -- deadline to finalize answers for this section
    CHECK (phase IN ('lobby', 'questions_open', 'section_review', 'section_score', 'final_winners')),
    CHECK ((phase IN ('question_open', 'question_closed')) = (current_question_id IS NOT NULL))
) STRICT;
