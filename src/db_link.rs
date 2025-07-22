use rusqlite:: {params,Connection, Result};

pub struct DbEntry{
    pub url : String,
    pub yt_id : String,
    pub friendly_name : String,
    pub real_name : String,
    pub timestamp : String,
    pub ip : String,
}

pub fn init() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("history_of_download.sqlite")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS entries (
            url TEXT,
            yt_id TEXT,
            friendly_name TEXT,
            real_name TEXT,
            timestamp TEXT,
            ip TEXT
        )",
        [],
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn add_entry(entry:DbEntry) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("history_of_download.sqlite")?;

    conn.execute(
        "INSERT INTO entries (url, yt_id, friendly_name, real_name, timestamp, ip) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            entry.url,
            entry.yt_id,
            entry.friendly_name,
            entry.real_name,
            entry.timestamp,
            entry.ip
        ],
    )?;
    Ok(())
}