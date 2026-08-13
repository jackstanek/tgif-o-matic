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
    created_by    INTEGER NOT NULL REFERENCES users(id),
    created_at    INTEGER NOT NULL
) STRICT;

-- Information about the administrators, who can create templates and run game instances.
CREATE TABLE admins (
    id            INTEGER PRIMARY KEY,
    username      TEXT NOT NULL,
    pw_hash       TEXT NOT NULL,
    pw_salt       TEXT NOT NULL
) STRICT;

-- Information about players. Each player is ephemeral and per-game (no
-- persistence across games for now).
CREATE TABLE players (
    id            INTEGER PRIMARY KEY,
    game_id       INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    nickname      TEXT NOT NULL,
    created_at    INTEGER NOT NULL
) STRICT;

-- Information about the teams
CREATE TABLE teams (
    id            INTEGER PRIMARY KEY,
    game_id       INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    captain       INTEGER NOT NULL REFERENCES players(id), -- TODO: on delete semantics
    join_code     BLOB NOT NULL,
    team_name     TEXT NOT NULL UNIQUE,
    created_at    INTEGER NOT NULL
) STRICT;

-- Team membership mappings
CREATE TABLE player_teams (
    player_id     INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    team_id       INTEGER NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    PRIMARY KEY (player_id, team_id)
) STRICT;

-- Ephemeral player sessions. Each session represents a player in a particular
-- game instance.
CREATE TABLE sessions (
    token_hash    BLOB PRIMARY KEY, -- SHA-256 hash of session token
    game_id       INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    -- note: exactly one of admin_id or player_id is not null
    admin_id      INTEGER REFERENCES admins(id),  -- null for player sessions
    player_id     INTEGER REFERENCES players(id), -- null for admin sessions
    created_at    INTEGER NOT NULL,
    expires_at    INTEGER NOT NULL
) STRICT;
