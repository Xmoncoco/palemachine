use rusqlite:: {params,Connection, Result};
use webauthn_rs::prelude::Passkey as WebauthnPasskey;
use uuid::Uuid;

#[cfg(not(test))]
const DB_PATH: &str = "history_of_download.sqlite";
#[cfg(test)]
const DB_PATH: &str = "test_history.sqlite";

pub struct DbEntry{
    pub url : String,
    pub yt_id : String,
    pub friendly_name : String,
    pub real_name : String,
    pub timestamp : String,
    pub ip : String,
}

#[derive(Debug, Clone)]
pub struct Passkey {
    pub user_id: Uuid,
    pub passkey: WebauthnPasskey,
}

pub fn init() -> Result<(), rusqlite::Error> {
    let conn = Connection::open(DB_PATH)?;

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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS passkeys (
            user_id TEXT,
            passkey_json TEXT
        )",
        [],
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn add_entry(entry:DbEntry) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(DB_PATH)?;

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

pub fn add_passkey(user_id: Uuid, passkey: &WebauthnPasskey) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(DB_PATH)?;
    let passkey_json = serde_json::to_string(passkey).unwrap();
    
    conn.execute(
        "INSERT INTO passkeys (user_id, passkey_json) VALUES (?1, ?2)",
        params![user_id.to_string(), passkey_json],
    )?;
    Ok(())
}

pub fn get_passkeys(user_id: Uuid) -> Result<Vec<WebauthnPasskey>, rusqlite::Error> {
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare("SELECT passkey_json FROM passkeys WHERE user_id = ?1")?;
    
    let passkeys_iter = stmt.query_map(params![user_id.to_string()], |row| {
        let json: String = row.get(0)?;
        let passkey: WebauthnPasskey = serde_json::from_str(&json).unwrap();
        Ok(passkey)
    })?;

    let mut passkeys = Vec::new();
    for passkey in passkeys_iter {
        passkeys.push(passkey?);
    }
    Ok(passkeys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn cleanup() {
        let _ = fs::remove_file(DB_PATH);
    }

    #[test]
    fn test_init_db() {
        cleanup();
        assert!(init().is_ok());
        assert!(std::path::Path::new(DB_PATH).exists());
        cleanup();
    }

    #[test]
    fn test_add_entry() {
        cleanup();
        init().unwrap();
        let entry = DbEntry {
            url: "http://example.com".to_string(),
            yt_id: "123".to_string(),
            friendly_name: "Test".to_string(),
            real_name: "Real Test".to_string(),
            timestamp: "2023-01-01".to_string(),
            ip: "127.0.0.1".to_string(),
        };
        assert!(add_entry(entry).is_ok());
        cleanup();
    }

    #[test]
    fn test_passkeys() {
        cleanup();
        init().unwrap();
        let user_id = Uuid::new_v4();
        // Mocking a WebauthnPasskey is hard because it has private fields or complex structure.
        // However, we can test that get_passkeys returns empty initially.
        let passkeys = get_passkeys(user_id).unwrap();
        assert!(passkeys.is_empty());
        
        // We can't easily create a valid WebauthnPasskey without the webauthn-rs library's internal logic or a valid registration ceremony.
        // So we will skip adding a passkey in this simple unit test unless we mock it, but the struct fields might be accessible.
        // Looking at webauthn-rs docs (or source if I could), Passkey struct usually comes from registration.
        
        cleanup();
    }
}