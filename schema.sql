-- ============================================================================
-- Index Stream — Core Video Schema (SQLite)
-- Tailored to video content with items/versions/parts and JSON metadata.
-- ============================================================================

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

BEGIN;

-- ----------------------------------------------------------------------------
-- TOKENS - authentication tokens
-- Columns:
--   token       : text key
--   user_agent  : user's browser information
--   created_at  : epoch seconds
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS tokens (
  token            TEXT PRIMARY KEY,
  user_agent       TEXT,
  created_at       INTEGER NOT NULL
);

-- ----------------------------------------------------------------------------
-- PROFILES - user profiles
-- Columns:
--   id          : autoincrement surrogate key
--   name        : display name (e.g., "Vids")
--   color       : hex color code (e.g., "#000000")
--   created_at  : epoch seconds
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS profiles (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  name             TEXT NOT NULL,
  color            TEXT NOT NULL,
  created_at       INTEGER NOT NULL
);

-- ----------------------------------------------------------------------------
-- INDEXES (logical groupings; replaces prior "libraries")
-- Columns:
--   id          : autoincrement surrogate key
--   name        : display name (e.g., "Vids")
--   type        : "videos" | "photos" | "audio" (schema here focuses on videos)
--   is_plugin   : 0/1 — whether this index is provided by a plugin
--   icon        : UI hint (e.g., "movie")
--   created_at  : epoch seconds
--   metadata    : JSON bag (e.g., {"folders":["/path/a","/path/b"], ...})
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS indexes (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  name             TEXT NOT NULL,
  type             TEXT NOT NULL CHECK (type IN ('videos','photos','audio')),
  is_plugin        INTEGER NOT NULL DEFAULT 0,           -- boolean: 0 = false, 1 = true
  icon             TEXT,                                 -- e.g., "movie"
  created_at       INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  metadata         TEXT NOT NULL DEFAULT '{}'            -- JSON (folders, extra settings)
                     CHECK (json_valid(metadata))
);

-- ----------------------------------------------------------------------------
-- SCAN JOBS — scan jobs
-- status: 'queued' | 'scanning'
-- Columns:
--   id              : autoincrement surrogate key
--   index_id        : FK to indexes.id (scoping all queries)
--   status          : status of the scan job
--   created_at      : when THIS item was added (epoch seconds)
--   updated_at      : when THIS item was updated (epoch seconds)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS scan_jobs (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  index_id         INTEGER NOT NULL REFERENCES indexes(id) ON DELETE CASCADE,
  status           TEXT NOT NULL CHECK (status IN ('queued','scanning')),
  created_at       INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  updated_at       INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

-- ----------------------------------------------------------------------------
-- VIDEO ITEMS — semantic works and hierarchy
-- type: 'video' | 'movie' | 'show' | 'season' | 'episode'
-- Columns:
--   id              : autoincrement surrogate key
--   index_id        : FK to indexes.id (scoping all queries)
--   type            : what kind of node (movie/show/season/episode/other video)
--   parent_id       : hierarchy link (season->show, episode->season)
--   title           : human-facing title
--   sort_title      : normalized sort key (e.g., "Matrix, The")
--   year            : quick integer filter; full dates live in metadata JSON
--   number          : season or episode number depending on type
--   metadata        : JSON (provider IDs like tmdb_id/tvdb_id, aka titles, etc.)
--   added_at        : when THIS item was added (epoch seconds)
--   latest_added_at : max(added_at) for THIS item AND all descendants (bubble-up)
--   created_at/updated_at : bookkeeping
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS video_items (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  index_id         INTEGER NOT NULL REFERENCES indexes(id) ON DELETE CASCADE,
  type             TEXT NOT NULL CHECK (type IN ('video','movie','show','season','episode')),
  parent_id        INTEGER REFERENCES video_items(id) ON DELETE CASCADE,

  title            TEXT NOT NULL,
  sort_title       TEXT,
  year             INTEGER,

  number           INTEGER,                              -- season or episode number (context by type)

  metadata         TEXT NOT NULL DEFAULT '{}'            -- JSON payload (tmdb_id, tvdb_id, etc.)
                     CHECK (json_valid(metadata)),

  added_at         INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  latest_added_at  INTEGER NOT NULL DEFAULT (strftime('%s','now')),

  created_at       INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  updated_at       INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

-- Index strategy: scope by index_id for common lookups,
-- except parent lookup which benefits from a standalone index.
CREATE INDEX IF NOT EXISTS idx_video_items_index_type
  ON video_items(index_id, type);

CREATE INDEX IF NOT EXISTS idx_video_items_parent
  ON video_items(parent_id);

CREATE INDEX IF NOT EXISTS idx_video_items_index_title
  ON video_items(index_id, title);

CREATE INDEX IF NOT EXISTS idx_video_items_index_sort_title
  ON video_items(index_id, sort_title);

CREATE INDEX IF NOT EXISTS idx_video_items_index_year
  ON video_items(index_id, year);

CREATE INDEX IF NOT EXISTS idx_video_items_index_latest_added
  ON video_items(index_id, latest_added_at DESC);

-- ----------------------------------------------------------------------------
-- VIDEO VERSIONS — a specific digital release/encode of an item
-- Columns:
--   id            : autoincrement surrogate key
--   item_id       : FK to video_items.id
--   edition       : label ("Director's Cut", "4K Remux")
--   source        : origin ("bluray_rip", "web_dl", "dvd", "stream")
--   container     : wrapper format ("mkv","mp4","m2ts",...)
--   resolution    : "2160p","1080p", etc.
--   hdr           : 0/1 flag if HDR detected
--   audio_channels: num channels (2,6,8)
--   bitrate       : overall bps (optional)
--   runtime_ms    : duration (ms); handy for UI
--   probe_version : version of analyzer used (e.g., "ffprobe-7.0")
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS video_versions (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  item_id          INTEGER NOT NULL REFERENCES video_items(id) ON DELETE CASCADE,

  edition          TEXT,
  source           TEXT,
  container        TEXT,
  resolution       TEXT,
  hdr              INTEGER DEFAULT 0,
  audio_channels   INTEGER,
  bitrate          INTEGER,
  runtime_ms       INTEGER,
  probe_version    TEXT,

  created_at       INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  updated_at       INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

-- Original index (as requested)
CREATE INDEX IF NOT EXISTS idx_video_versions_item
  ON video_versions(item_id);

-- ----------------------------------------------------------------------------
-- VIDEO PARTS — actual file(s) backing a version
-- Columns:
--   id          : autoincrement surrogate key
--   version_id  : FK to video_versions.id
--   path        : absolute or server-relative path (unique)
--   size        : file size (bytes)
--   mtime       : file mtime (epoch seconds)
--   part_index  : playback order within the version
--   duration_ms : per-file duration (ms)
--   fast_hash   : cheap content signature (e.g., SHA1(first 4–8KiB))
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS video_parts (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  version_id       INTEGER NOT NULL REFERENCES video_versions(id) ON DELETE CASCADE,

  path             TEXT NOT NULL,
  size             INTEGER,
  mtime            INTEGER,
  part_index       INTEGER NOT NULL DEFAULT 0,
  duration_ms      INTEGER,
  fast_hash        TEXT,

  created_at       INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  updated_at       INTEGER NOT NULL DEFAULT (strftime('%s','now')),

  UNIQUE(path)
);

-- Original indexes (as requested)
CREATE INDEX IF NOT EXISTS idx_video_parts_version_order
  ON video_parts(version_id, part_index ASC);

CREATE INDEX IF NOT EXISTS idx_video_parts_fast_sig
  ON video_parts(size, fast_hash);

-- ----------------------------------------------------------------------------
-- TOUCH & BUBBLE TRIGGERS
-- Keep updated_at fresh; bubble increases of latest_added_at up the tree.
-- ----------------------------------------------------------------------------

-- Touch updated_at on direct updates (if caller didn't set it)
CREATE TRIGGER IF NOT EXISTS trg_video_items_touch_upd
AFTER UPDATE ON video_items
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
  UPDATE video_items SET updated_at = strftime('%s','now')
  WHERE id = NEW.id;
END;

-- Bubble child's latest_added_at to parent on INSERT
CREATE TRIGGER IF NOT EXISTS trg_video_items_child_insert_bubble
AFTER INSERT ON video_items
FOR EACH ROW
WHEN NEW.parent_id IS NOT NULL
BEGIN
  UPDATE video_items
     SET latest_added_at = MAX(COALESCE(latest_added_at, added_at), NEW.latest_added_at),
         updated_at      = strftime('%s','now')
   WHERE id = NEW.parent_id
     AND NEW.latest_added_at > COALESCE(
           (SELECT latest_added_at FROM video_items WHERE id = NEW.parent_id),
           (SELECT added_at       FROM video_items WHERE id = NEW.parent_id)
         );
END;

-- Bubble upward when a row's latest_added_at increases
CREATE TRIGGER IF NOT EXISTS trg_video_items_bubble_up_on_increase
AFTER UPDATE OF latest_added_at ON video_items
FOR EACH ROW
WHEN NEW.parent_id IS NOT NULL AND NEW.latest_added_at > OLD.latest_added_at
BEGIN
  UPDATE video_items
     SET latest_added_at = MAX(COALESCE(latest_added_at, added_at), NEW.latest_added_at),
         updated_at      = strftime('%s','now')
   WHERE id = NEW.parent_id
     AND NEW.latest_added_at > COALESCE(
           (SELECT latest_added_at FROM video_items WHERE id = NEW.parent_id),
           (SELECT added_at       FROM video_items WHERE id = NEW.parent_id)
         );
END;

COMMIT;
