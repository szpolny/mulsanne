//! Local lap database (SQLite). Telemetry traces are stored as gzipped JSON
//! column arrays, one blob per lap.

use std::io::{Read, Write};
use std::path::Path;
use std::sync::Mutex;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::online::LapUpload;

pub type DbResult<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Per-lap telemetry, one entry per sample in every column.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LapTrace {
    /// Time into lap (s).
    pub t: Vec<f32>,
    /// Lap distance (m).
    pub d: Vec<f32>,
    /// m/s
    pub speed: Vec<f32>,
    pub throttle: Vec<f32>,
    pub brake: Vec<f32>,
    pub steering: Vec<f32>,
    pub clutch: Vec<f32>,
    pub gear: Vec<i8>,
    pub rpm: Vec<f32>,
    pub x: Vec<f32>,
    pub z: Vec<f32>,
    pub g_lat: Vec<f32>,
    pub g_long: Vec<f32>,
    pub abs: Vec<u8>,
    pub tc: Vec<u8>,
    /// Average surface temperature per tyre (°C).
    pub tyre_fl: Vec<f32>,
    pub tyre_fr: Vec<f32>,
    pub tyre_rl: Vec<f32>,
    pub tyre_rr: Vec<f32>,
}

impl LapTrace {
    pub fn len(&self) -> usize {
        self.t.len()
    }

    fn encode(&self) -> DbResult<Vec<u8>> {
        let json = serde_json::to_vec(self).map_err(err)?;
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&json).map_err(err)?;
        enc.finish().map_err(err)
    }

    pub fn decode(blob: &[u8]) -> DbResult<Self> {
        let mut json = Vec::new();
        GzDecoder::new(blob).read_to_end(&mut json).map_err(err)?;
        serde_json::from_slice(&json).map_err(err)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub id: i64,
    pub started_at: i64,
    pub source: String,
    pub track: String,
    pub track_length: f64,
    pub session_type: String,
    pub car: String,
    pub vehicle_name: String,
    pub car_class: String,
    pub driver: String,
    /// LMU build number; `None` for demo data and sessions recorded before it was stored.
    pub game_version: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    #[serde(flatten)]
    pub session: SessionRecord,
    pub lap_count: i64,
    pub valid_laps: i64,
    pub best_lap: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LapRecord {
    pub id: i64,
    pub session_id: i64,
    pub lap_number: i32,
    pub lap_time: f64,
    pub s1: Option<f64>,
    pub s2: Option<f64>,
    pub s3: Option<f64>,
    pub valid: bool,
    pub out_lap: bool,
    pub in_lap: bool,
    pub fuel_used: Option<f64>,
    pub ve_used: Option<f64>,
    pub max_speed: f64,
    pub track: String,
    pub car: String,
    pub car_class: String,
    pub recorded_at: i64,
    pub session_type: String,
}

pub struct NewLap {
    pub session: SessionRecord,
    pub lap_number: i32,
    pub lap_time: f64,
    pub sectors: [Option<f64>; 3],
    pub valid: bool,
    pub out_lap: bool,
    pub in_lap: bool,
    pub fuel_used: Option<f64>,
    pub ve_used: Option<f64>,
    pub recorded_at: i64,
    pub trace: LapTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackMap {
    pub track: String,
    pub length: f64,
    /// Centre line as `[x, z]` world coordinates, ~5 m spacing.
    pub points: Vec<[f32; 2]>,
    /// Lap distance of each point.
    pub dist: Vec<f32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalBest {
    pub track: String,
    pub car: String,
    pub car_class: String,
    pub lap_id: i64,
    pub lap_time: f64,
    pub s1: Option<f64>,
    pub s2: Option<f64>,
    pub s3: Option<f64>,
    pub recorded_at: i64,
    pub lap_count: i64,
    /// Recorded in the game, not demo data; only these can go online.
    pub from_game: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    pub tracks: Vec<String>,
    pub cars: Vec<String>,
}

pub struct Db {
    conn: Mutex<Connection>,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id            INTEGER PRIMARY KEY,
    started_at    INTEGER NOT NULL,
    source        TEXT NOT NULL,
    track         TEXT NOT NULL,
    track_length  REAL NOT NULL,
    session_type  TEXT NOT NULL,
    car           TEXT NOT NULL,
    vehicle_name  TEXT NOT NULL,
    car_class     TEXT NOT NULL,
    driver        TEXT NOT NULL,
    game_version  INTEGER
);
CREATE TABLE IF NOT EXISTS laps (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id   INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    lap_number   INTEGER NOT NULL,
    lap_time     REAL NOT NULL,
    s1           REAL,
    s2           REAL,
    s3           REAL,
    valid        INTEGER NOT NULL,
    out_lap      INTEGER NOT NULL,
    in_lap       INTEGER NOT NULL,
    fuel_used    REAL,
    ve_used      REAL,
    max_speed    REAL NOT NULL,
    track        TEXT NOT NULL,
    car          TEXT NOT NULL,
    car_class    TEXT NOT NULL,
    recorded_at  INTEGER NOT NULL,
    sample_count INTEGER NOT NULL,
    trace        BLOB NOT NULL
);
CREATE INDEX IF NOT EXISTS laps_session ON laps(session_id);
CREATE INDEX IF NOT EXISTS laps_track_car ON laps(track, car, valid, lap_time);
CREATE TABLE IF NOT EXISTS track_maps (
    track   TEXT PRIMARY KEY,
    length  REAL NOT NULL,
    lap_id  INTEGER,
    data    TEXT NOT NULL
);
-- Laps queued for or sent to the online rankings.
CREATE TABLE IF NOT EXISTS lap_uploads (
    lap_id      INTEGER PRIMARY KEY REFERENCES laps(id) ON DELETE CASCADE,
    status      TEXT NOT NULL,      -- queued | ranked | held | failed | rejected
    remote_id   TEXT,
    anonymous   INTEGER NOT NULL,
    attempts    INTEGER NOT NULL DEFAULT 0,
    error       TEXT,
    updated_at  INTEGER NOT NULL,
    -- Telemetry: NULL (not sent) | queued | uploaded | replaced (by a faster
    -- lap's) | skipped (not the best lap) | rejected
    trace       TEXT
);
"#;

/// Laps that may go to the online rankings: clean flying laps from the game
/// (never demo data) with all three sectors. Needs `laps l JOIN sessions s`.
const UPLOADABLE: &str = "l.valid = 1 AND l.out_lap = 0 AND l.in_lap = 0 AND s.source = 'lmu'
    AND l.s1 IS NOT NULL AND l.s2 IS NOT NULL AND l.s3 IS NOT NULL";

/// Failed uploads are retried this many times, with exponential backoff.
const MAX_UPLOAD_ATTEMPTS: i64 = 5;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadState {
    pub lap_id: i64,
    pub status: String,
    pub remote_id: Option<String>,
    pub error: Option<String>,
    /// Telemetry upload state; see the `lap_uploads` schema.
    pub trace: Option<String>,
}

/// A queued telemetry upload: the lap's stored trace blob (gzipped JSON).
pub struct PendingTrace {
    pub lap_id: i64,
    pub remote_id: String,
    pub blob: Vec<u8>,
}

/// A queued upload, ready to send.
pub struct PendingUpload {
    pub lap_id: i64,
    pub lap: LapUpload,
}

fn add_column(conn: &Connection, table: &str, column: &str, decl: &str) -> DbResult<()> {
    let exists = conn
        .prepare("SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2")
        .and_then(|mut s| s.exists(params![table, column]))
        .map_err(err)?;
    if !exists {
        conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {decl}")).map_err(err)?;
    }
    Ok(())
}

/// A lap counts towards personal bests when it's clean and a full flying lap.
const PB_LAP: &str = "valid = 1 AND out_lap = 0 AND in_lap = 0";

impl Db {
    pub fn open(path: &Path) -> DbResult<Self> {
        let conn = Connection::open(path).map_err(err)?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;").map_err(err)?;
        conn.execute_batch(SCHEMA).map_err(err)?;
        // Columns added after their table; SCHEMA only creates missing tables.
        add_column(&conn, "sessions", "game_version", "INTEGER")?;
        add_column(&conn, "lap_uploads", "trace", "TEXT")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn insert_lap(&self, lap: &NewLap) -> DbResult<i64> {
        let blob = lap.trace.encode()?;
        let max_speed = lap.trace.speed.iter().cloned().fold(0.0f32, f32::max) as f64;
        let s = &lap.session;
        let conn = self.conn();
        conn.execute(
            "INSERT OR IGNORE INTO sessions (id, started_at, source, track, track_length, session_type, car, vehicle_name, car_class, driver, game_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![s.id, s.started_at, s.source, s.track, s.track_length, s.session_type, s.car, s.vehicle_name, s.car_class, s.driver, s.game_version],
        )
        .map_err(err)?;
        conn.execute(
            "INSERT INTO laps (session_id, lap_number, lap_time, s1, s2, s3, valid, out_lap, in_lap, fuel_used, ve_used, max_speed, track, car, car_class, recorded_at, sample_count, trace)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                s.id,
                lap.lap_number,
                lap.lap_time,
                lap.sectors[0],
                lap.sectors[1],
                lap.sectors[2],
                lap.valid,
                lap.out_lap,
                lap.in_lap,
                lap.fuel_used,
                lap.ve_used,
                max_speed,
                s.track,
                s.car,
                s.car_class,
                lap.recorded_at,
                lap.trace.len() as i64,
                blob
            ],
        )
        .map_err(err)?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_sessions(&self, track: Option<&str>, car: Option<&str>, limit: i64, offset: i64) -> DbResult<Vec<SessionSummary>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.started_at, s.source, s.track, s.track_length, s.session_type, s.car, s.vehicle_name, s.car_class, s.driver, s.game_version,
                        COUNT(l.id), COALESCE(SUM(l.valid), 0),
                        MIN(CASE WHEN l.valid = 1 AND l.out_lap = 0 AND l.in_lap = 0 THEN l.lap_time END)
                 FROM sessions s LEFT JOIN laps l ON l.session_id = s.id
                 WHERE (?1 IS NULL OR s.track = ?1) AND (?2 IS NULL OR s.car = ?2)
                 GROUP BY s.id ORDER BY s.started_at DESC LIMIT ?3 OFFSET ?4",
            )
            .map_err(err)?;
        let rows = stmt
            .query_map(params![track, car, limit, offset], |r| {
                Ok(SessionSummary {
                    session: SessionRecord {
                        id: r.get(0)?,
                        started_at: r.get(1)?,
                        source: r.get(2)?,
                        track: r.get(3)?,
                        track_length: r.get(4)?,
                        session_type: r.get(5)?,
                        car: r.get(6)?,
                        vehicle_name: r.get(7)?,
                        car_class: r.get(8)?,
                        driver: r.get(9)?,
                        game_version: r.get(10)?,
                    },
                    lap_count: r.get(11)?,
                    valid_laps: r.get(12)?,
                    best_lap: r.get(13)?,
                })
            })
            .map_err(err)?;
        rows.collect::<Result<_, _>>().map_err(err)
    }

    const LAP_COLUMNS: &'static str = "l.id, l.session_id, l.lap_number, l.lap_time, l.s1, l.s2, l.s3, l.valid, l.out_lap, l.in_lap,
         l.fuel_used, l.ve_used, l.max_speed, l.track, l.car, l.car_class, l.recorded_at, s.session_type";

    fn map_lap(r: &rusqlite::Row) -> rusqlite::Result<LapRecord> {
        Ok(LapRecord {
            id: r.get(0)?,
            session_id: r.get(1)?,
            lap_number: r.get(2)?,
            lap_time: r.get(3)?,
            s1: r.get(4)?,
            s2: r.get(5)?,
            s3: r.get(6)?,
            valid: r.get(7)?,
            out_lap: r.get(8)?,
            in_lap: r.get(9)?,
            fuel_used: r.get(10)?,
            ve_used: r.get(11)?,
            max_speed: r.get(12)?,
            track: r.get(13)?,
            car: r.get(14)?,
            car_class: r.get(15)?,
            recorded_at: r.get(16)?,
            session_type: r.get(17)?,
        })
    }

    pub fn list_laps(&self, session_id: i64) -> DbResult<Vec<LapRecord>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM laps l JOIN sessions s ON s.id = l.session_id WHERE l.session_id = ?1 ORDER BY l.lap_number",
                Self::LAP_COLUMNS
            ))
            .map_err(err)?;
        let rows = stmt.query_map(params![session_id], Self::map_lap).map_err(err)?;
        rows.collect::<Result<_, _>>().map_err(err)
    }

    /// Laps on a track (optionally for one car), fastest first.
    pub fn list_track_laps(&self, track: &str, car: Option<&str>, valid_only: bool, limit: i64) -> DbResult<Vec<LapRecord>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM laps l JOIN sessions s ON s.id = l.session_id
                 WHERE l.track = ?1 AND (?2 IS NULL OR l.car = ?2) AND (?3 = 0 OR (l.valid = 1 AND l.out_lap = 0 AND l.in_lap = 0))
                 ORDER BY l.lap_time LIMIT ?4",
                Self::LAP_COLUMNS
            ))
            .map_err(err)?;
        let rows = stmt.query_map(params![track, car, valid_only, limit], Self::map_lap).map_err(err)?;
        rows.collect::<Result<_, _>>().map_err(err)
    }

    pub fn lap_trace(&self, lap_id: i64) -> DbResult<LapTrace> {
        let blob: Vec<u8> = self
            .conn()
            .query_row("SELECT trace FROM laps WHERE id = ?1", params![lap_id], |r| r.get(0))
            .map_err(err)?;
        LapTrace::decode(&blob)
    }

    /// Best clean lap for a track/car: `(lap_id, lap_time, sectors)`.
    pub fn best_lap(&self, track: &str, car: &str) -> DbResult<Option<(i64, f64, [Option<f64>; 3])>> {
        self.conn()
            .query_row(
                &format!("SELECT id, lap_time, s1, s2, s3 FROM laps WHERE track = ?1 AND car = ?2 AND {PB_LAP} ORDER BY lap_time LIMIT 1"),
                params![track, car],
                |r| Ok((r.get(0)?, r.get(1)?, [r.get(2)?, r.get(3)?, r.get(4)?])),
            )
            .optional()
            .map_err(err)
    }

    pub fn personal_bests(&self) -> DbResult<Vec<PersonalBest>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT l.track, l.car, l.car_class, l.id, l.lap_time, l.s1, l.s2, l.s3, l.recorded_at, c.n,
                        (SELECT source FROM sessions WHERE id = l.session_id) = 'lmu'
                 FROM laps l
                 JOIN (SELECT track, car, MIN(lap_time) AS best, COUNT(*) AS n FROM laps WHERE {PB_LAP} GROUP BY track, car) c
                   ON c.track = l.track AND c.car = l.car AND c.best = l.lap_time
                 WHERE l.valid = 1 AND l.out_lap = 0 AND l.in_lap = 0
                 GROUP BY l.track, l.car
                 ORDER BY l.track, l.lap_time"
            ))
            .map_err(err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok(PersonalBest {
                    track: r.get(0)?,
                    car: r.get(1)?,
                    car_class: r.get(2)?,
                    lap_id: r.get(3)?,
                    lap_time: r.get(4)?,
                    s1: r.get(5)?,
                    s2: r.get(6)?,
                    s3: r.get(7)?,
                    recorded_at: r.get(8)?,
                    lap_count: r.get(9)?,
                    from_game: r.get(10)?,
                })
            })
            .map_err(err)?;
        rows.collect::<Result<_, _>>().map_err(err)
    }

    pub fn filters(&self) -> DbResult<Filters> {
        let conn = self.conn();
        let collect = |sql: &str| -> DbResult<Vec<String>> {
            let mut stmt = conn.prepare(sql).map_err(err)?;
            let rows = stmt.query_map([], |r| r.get(0)).map_err(err)?;
            rows.collect::<Result<_, _>>().map_err(err)
        };
        Ok(Filters {
            tracks: collect("SELECT DISTINCT track FROM sessions ORDER BY track")?,
            cars: collect("SELECT DISTINCT car FROM sessions ORDER BY car")?,
        })
    }

    pub fn delete_lap(&self, lap_id: i64) -> DbResult<()> {
        self.conn().execute("DELETE FROM laps WHERE id = ?1", params![lap_id]).map_err(err)?;
        Ok(())
    }

    pub fn delete_session(&self, session_id: i64) -> DbResult<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM laps WHERE session_id = ?1", params![session_id]).map_err(err)?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id]).map_err(err)?;
        Ok(())
    }

    pub fn track_map(&self, track: &str) -> DbResult<Option<TrackMap>> {
        let data: Option<String> = self
            .conn()
            .query_row("SELECT data FROM track_maps WHERE track = ?1", params![track], |r| r.get(0))
            .optional()
            .map_err(err)?;
        data.map(|d| serde_json::from_str(&d).map_err(err)).transpose()
    }

    pub fn save_track_map(&self, map: &TrackMap, lap_id: i64) -> DbResult<()> {
        let data = serde_json::to_string(map).map_err(err)?;
        self.conn()
            .execute(
                "INSERT INTO track_maps (track, length, lap_id, data) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(track) DO UPDATE SET length = excluded.length, lap_id = excluded.lap_id, data = excluded.data",
                params![map.track, map.length, lap_id, data],
            )
            .map_err(err)?;
        Ok(())
    }

    /// Queues uploadable laps among `lap_ids`; failed or rejected ones are
    /// queued again. Returns how many were queued.
    pub fn enqueue_uploads(&self, lap_ids: &[i64], anonymous: bool, now: i64) -> DbResult<usize> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "INSERT INTO lap_uploads (lap_id, status, anonymous, updated_at)
                 SELECT l.id, 'queued', ?2, ?3 FROM laps l JOIN sessions s ON s.id = l.session_id
                 WHERE l.id = ?1 AND {UPLOADABLE}
                 ON CONFLICT(lap_id) DO UPDATE SET status = 'queued', attempts = 0, error = NULL, trace = NULL,
                     anonymous = excluded.anonymous, updated_at = excluded.updated_at
                 WHERE lap_uploads.status IN ('failed', 'rejected')"
            ))
            .map_err(err)?;
        let mut queued = 0;
        for id in lap_ids {
            queued += stmt.execute(params![id, anonymous, now]).map_err(err)?;
        }
        Ok(queued)
    }

    /// The next upload to send: queued ones first, then failed ones whose
    /// backoff (1, 2, 4, … min) has passed.
    pub fn next_upload(&self, now: i64, app_version: &str) -> DbResult<Option<PendingUpload>> {
        self.conn()
            .query_row(
                &format!(
                    "SELECT u.lap_id, u.anonymous, s.track, s.track_length, s.car, s.vehicle_name, s.car_class, s.session_type, s.game_version,
                            l.lap_time, l.s1, l.s2, l.s3, l.fuel_used, l.ve_used, l.max_speed, l.recorded_at
                     FROM lap_uploads u JOIN laps l ON l.id = u.lap_id JOIN sessions s ON s.id = l.session_id
                     WHERE {UPLOADABLE} AND (u.status = 'queued'
                        OR (u.status = 'failed' AND u.attempts < ?2 AND u.updated_at + 60 * (1 << (u.attempts - 1)) <= ?1))
                     ORDER BY u.status = 'failed', u.updated_at LIMIT 1"
                ),
                params![now, MAX_UPLOAD_ATTEMPTS],
                |r| {
                    Ok(PendingUpload {
                        lap_id: r.get(0)?,
                        lap: LapUpload {
                            anonymous: r.get(1)?,
                            track: r.get(2)?,
                            track_length: r.get(3)?,
                            car: r.get(4)?,
                            vehicle_name: r.get(5)?,
                            car_class: r.get(6)?,
                            session_type: r.get(7)?,
                            game_version: r.get(8)?,
                            lap_time: r.get(9)?,
                            s1: r.get(10)?,
                            s2: r.get(11)?,
                            s3: r.get(12)?,
                            fuel_used: r.get(13)?,
                            ve_used: r.get(14)?,
                            max_speed: r.get(15)?,
                            recorded_at: r.get(16)?,
                            app_version: app_version.to_owned(),
                        },
                    })
                },
            )
            .optional()
            .map_err(err)
    }

    /// Records an upload attempt. `failed` counts towards the retry limit.
    pub fn set_upload_status(
        &self,
        lap_id: i64,
        status: &str,
        remote_id: Option<&str>,
        error: Option<&str>,
        now: i64,
    ) -> DbResult<()> {
        self.conn()
            .execute(
                "UPDATE lap_uploads SET status = ?2, remote_id = COALESCE(?3, remote_id), error = ?4, updated_at = ?5,
                     attempts = attempts + (?2 = 'failed')
                 WHERE lap_id = ?1",
                params![lap_id, status, remote_id, error, now],
            )
            .map_err(err)?;
        Ok(())
    }

    pub fn upload_states(&self) -> DbResult<Vec<UploadState>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT lap_id, status, remote_id, error, trace FROM lap_uploads").map_err(err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok(UploadState { lap_id: r.get(0)?, status: r.get(1)?, remote_id: r.get(2)?, error: r.get(3)?, trace: r.get(4)? })
            })
            .map_err(err)?;
        rows.collect::<Result<_, _>>().map_err(err)
    }

    /// Queues the telemetry of an uploaded lap if it's the fastest uploaded
    /// lap on its track and class (the server keeps only that one). Returns
    /// whether it was queued.
    pub fn queue_trace_if_best(&self, lap_id: i64) -> DbResult<bool> {
        let queued = self
            .conn()
            .execute(
                "UPDATE lap_uploads SET trace = 'queued' WHERE lap_id = ?1 AND remote_id IS NOT NULL
                   AND NOT EXISTS (
                     SELECT 1 FROM lap_uploads o JOIN laps ol ON ol.id = o.lap_id, laps l
                     WHERE l.id = ?1 AND o.lap_id <> ?1 AND o.status IN ('ranked', 'held')
                       AND ol.track = l.track AND ol.car_class = l.car_class
                       AND (ol.lap_time < l.lap_time OR (ol.lap_time = l.lap_time AND ol.recorded_at < l.recorded_at)))",
                params![lap_id],
            )
            .map_err(err)?;
        Ok(queued > 0)
    }

    pub fn next_trace(&self) -> DbResult<Option<PendingTrace>> {
        self.conn()
            .query_row(
                "SELECT u.lap_id, u.remote_id, l.trace FROM lap_uploads u JOIN laps l ON l.id = u.lap_id
                 WHERE u.trace = 'queued' AND u.remote_id IS NOT NULL ORDER BY u.updated_at LIMIT 1",
                [],
                |r| Ok(PendingTrace { lap_id: r.get(0)?, remote_id: r.get(1)?, blob: r.get(2)? }),
            )
            .optional()
            .map_err(err)
    }

    /// Records a telemetry upload attempt. A successful one supersedes the
    /// traces of slower laps on the same track and class, which the server
    /// drops.
    pub fn set_trace_status(&self, lap_id: i64, status: &str, error: Option<&str>) -> DbResult<()> {
        let conn = self.conn();
        conn.execute("UPDATE lap_uploads SET trace = ?2, error = ?3 WHERE lap_id = ?1", params![lap_id, status, error])
            .map_err(err)?;
        if status == "uploaded" {
            conn.execute(
                "UPDATE lap_uploads SET trace = 'replaced'
                 WHERE trace = 'uploaded' AND lap_id <> ?1 AND lap_id IN (
                     SELECT o.id FROM laps o, laps l
                     WHERE l.id = ?1 AND o.track = l.track AND o.car_class = l.car_class)",
                params![lap_id],
            )
            .map_err(err)?;
        }
        Ok(())
    }

    pub fn upload_remote_id(&self, lap_id: i64) -> DbResult<Option<String>> {
        self.conn()
            .query_row("SELECT remote_id FROM lap_uploads WHERE lap_id = ?1", params![lap_id], |r| r.get(0))
            .optional()
            .map(Option::flatten)
            .map_err(err)
    }

    pub fn remove_upload(&self, lap_id: i64) -> DbResult<()> {
        self.conn().execute("DELETE FROM lap_uploads WHERE lap_id = ?1", params![lap_id]).map_err(err)?;
        Ok(())
    }

    /// Forgets every upload, after the user deleted their online data.
    pub fn clear_uploads(&self) -> DbResult<()> {
        self.conn().execute("DELETE FROM lap_uploads", []).map_err(err)?;
        Ok(())
    }
}

/// Builds a ~`spacing` m resampled centre line from a lap's positions.
pub fn track_map_from_trace(track: &str, length: f64, trace: &LapTrace, spacing: f32) -> TrackMap {
    let mut points = Vec::new();
    let mut dist = Vec::new();
    let mut last_d = f32::NEG_INFINITY;
    for i in 0..trace.len() {
        if trace.d[i] - last_d >= spacing {
            points.push([trace.x[i], trace.z[i]]);
            dist.push(trace.d[i]);
            last_d = trace.d[i];
        }
    }
    TrackMap { track: track.to_string(), length, points, dist }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(id: i64) -> SessionRecord {
        SessionRecord {
            id,
            started_at: 1,
            source: "demo".into(),
            track: "T".into(),
            track_length: 1000.0,
            session_type: "Practice".into(),
            car: "Car".into(),
            vehicle_name: "#1 Car".into(),
            car_class: "GT3".into(),
            driver: "Me".into(),
            game_version: None,
        }
    }

    fn lap(n: i32, time: f64, valid: bool) -> NewLap {
        let mut trace = LapTrace::default();
        for i in 0..10 {
            trace.t.push(i as f32);
            trace.d.push(i as f32 * 100.0);
            trace.speed.push(50.0 + i as f32);
            trace.x.push(i as f32);
            trace.z.push(0.0);
        }
        NewLap {
            session: session(7),
            lap_number: n,
            lap_time: time,
            sectors: [Some(30.0), Some(30.0), Some(time - 60.0)],
            valid,
            out_lap: false,
            in_lap: false,
            fuel_used: Some(2.5),
            ve_used: None,
            recorded_at: 100 + n as i64,
            trace,
        }
    }

    #[test]
    fn roundtrip_laps_and_bests() {
        let dir = std::env::temp_dir().join(format!("lmo-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.db");
        let _ = std::fs::remove_file(&path);
        let db = Db::open(&path).unwrap();

        let a = db.insert_lap(&lap(1, 95.0, true)).unwrap();
        let _b = db.insert_lap(&lap(2, 93.0, false)).unwrap();
        let c = db.insert_lap(&lap(3, 94.0, true)).unwrap();

        let sessions = db.list_sessions(None, None, 10, 0).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].lap_count, 3);
        assert_eq!(sessions[0].best_lap, Some(94.0));

        assert_eq!(db.best_lap("T", "Car").unwrap().unwrap().0, c);
        let pbs = db.personal_bests().unwrap();
        assert_eq!(pbs.len(), 1);
        assert_eq!(pbs[0].lap_id, c);
        assert_eq!(pbs[0].lap_count, 2);

        let trace = db.lap_trace(a).unwrap();
        assert_eq!(trace.len(), 10);
        assert_eq!(trace.speed[3], 53.0);

        let map = track_map_from_trace("T", 1000.0, &trace, 150.0);
        assert_eq!(map.points.len(), 5);
        db.save_track_map(&map, a).unwrap();
        assert_eq!(db.track_map("T").unwrap().unwrap().points.len(), 5);

        db.delete_session(7).unwrap();
        assert!(db.list_sessions(None, None, 10, 0).unwrap().is_empty());
    }

    #[test]
    fn adds_game_version_to_old_databases() {
        let dir = std::env::temp_dir().join(format!("lmo-migrate-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.db");
        let _ = std::fs::remove_file(&path);
        Connection::open(&path)
            .unwrap()
            .execute_batch(
                "CREATE TABLE sessions (id INTEGER PRIMARY KEY, started_at INTEGER NOT NULL, source TEXT NOT NULL,
                     track TEXT NOT NULL, track_length REAL NOT NULL, session_type TEXT NOT NULL, car TEXT NOT NULL,
                     vehicle_name TEXT NOT NULL, car_class TEXT NOT NULL, driver TEXT NOT NULL);
                 INSERT INTO sessions VALUES (1, 1, 'lmu', 'T', 1000, 'Practice', 'Car', '#1 Car', 'GT3', 'Me');",
            )
            .unwrap();

        let db = Db::open(&path).unwrap();
        let sessions = db.list_sessions(None, None, 10, 0).unwrap();
        assert_eq!(sessions[0].session.game_version, None);
        // Opening again doesn't try to add the column twice.
        drop(db);
        Db::open(&path).unwrap();
    }

    #[test]
    fn trace_queue_keeps_the_best_lap() {
        let dir = std::env::temp_dir().join(format!("lmo-trace-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.db");
        let _ = std::fs::remove_file(&path);
        let db = Db::open(&path).unwrap();
        let uploaded = |n: i32, time: f64| {
            let mut l = lap(n, time, true);
            l.session = SessionRecord { id: 9, source: "lmu".into(), ..session(9) };
            let id = db.insert_lap(&l).unwrap();
            db.enqueue_uploads(&[id], false, 1000).unwrap();
            db.set_upload_status(id, "ranked", Some(&format!("r{n}")), None, 1000).unwrap();
            id
        };

        let first = uploaded(1, 95.0);
        assert!(db.queue_trace_if_best(first).unwrap());
        let pending = db.next_trace().unwrap().unwrap();
        assert_eq!((pending.lap_id, pending.remote_id.as_str()), (first, "r1"));
        assert_eq!(LapTrace::decode(&pending.blob).unwrap().len(), 10);
        db.set_trace_status(first, "uploaded", None).unwrap();

        // Slower: not queued.
        let slower = uploaded(2, 96.0);
        assert!(!db.queue_trace_if_best(slower).unwrap());
        // Faster: queued, and its upload replaces the first lap's.
        let faster = uploaded(3, 94.0);
        assert!(db.queue_trace_if_best(faster).unwrap());
        db.set_trace_status(faster, "uploaded", None).unwrap();
        assert!(db.next_trace().unwrap().is_none());
        let trace = |id| db.upload_states().unwrap().into_iter().find(|s| s.lap_id == id).unwrap().trace;
        assert_eq!(trace(first).as_deref(), Some("replaced"));
        assert_eq!(trace(slower), None);
        assert_eq!(trace(faster).as_deref(), Some("uploaded"));
    }

    #[test]
    fn upload_queue() {
        let dir = std::env::temp_dir().join(format!("lmo-upload-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.db");
        let _ = std::fs::remove_file(&path);
        let db = Db::open(&path).unwrap();

        let demo = db.insert_lap(&lap(1, 95.0, true)).unwrap();
        let game_lap = |n: i32, time: f64, valid: bool| {
            let mut l = lap(n, time, valid);
            l.session = SessionRecord { id: 8, source: "lmu".into(), game_version: Some(1123), ..session(8) };
            db.insert_lap(&l).unwrap()
        };
        let clean = game_lap(2, 94.0, true);
        let invalid = game_lap(3, 93.0, false);

        // Demo and invalid laps never qualify.
        assert_eq!(db.enqueue_uploads(&[demo, clean, invalid], true, 1000).unwrap(), 1);
        // Already queued: not counted again.
        assert_eq!(db.enqueue_uploads(&[clean], true, 1000).unwrap(), 0);

        let next = db.next_upload(1000, "0.5.0").unwrap().unwrap();
        assert_eq!(next.lap_id, clean);
        assert!(next.lap.anonymous);
        assert_eq!(next.lap.s3, 34.0);
        assert_eq!(next.lap.app_version, "0.5.0");
        assert_eq!(next.lap.game_version, Some(1123));

        // A failure waits for its backoff before it's retried.
        db.set_upload_status(clean, "failed", None, Some("offline"), 1000).unwrap();
        assert!(db.next_upload(1030, "0.5.0").unwrap().is_none());
        assert!(db.next_upload(1060, "0.5.0").unwrap().is_some());

        db.set_upload_status(clean, "ranked", Some("abc"), None, 1100).unwrap();
        assert!(db.next_upload(10_000, "0.5.0").unwrap().is_none());
        assert_eq!(db.upload_remote_id(clean).unwrap().as_deref(), Some("abc"));
        // Uploaded laps aren't queued again.
        assert_eq!(db.enqueue_uploads(&[clean], false, 1200).unwrap(), 0);

        let states = db.upload_states().unwrap();
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].status, "ranked");

        // Deleting the lap locally drops its upload record.
        db.delete_lap(clean).unwrap();
        assert!(db.upload_states().unwrap().is_empty());
    }
}
