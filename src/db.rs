use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum PermissionState {
    AllowOnce,
    Always,
    Block,
}

impl ToString for PermissionState {
    fn to_string(&self) -> String {
        match self {
            Self::AllowOnce => "Allow Once".to_string(),
            Self::Always => "Always".to_string(),
            Self::Block => "Block".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionEntry {
    pub app_name: String,
    pub requested_permissions: Vec<String>,
    pub permission_state: PermissionState,
}

pub struct PermissionDB {
    conn: Connection,
}

impl PermissionDB {
    pub fn init() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("permission_manager");
        std::fs::create_dir_all(&data_dir)?;
        
        let conn = Connection::open(data_dir.join("permissions.db"))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS permissions (
                app_name TEXT PRIMARY KEY,
                requested_permissions TEXT NOT NULL,
                permission_state TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn get_all(&self) -> Result<Vec<PermissionEntry>> {
        let mut stmt = self.conn.prepare("SELECT * FROM permissions")?;
        let rows = stmt.query_map([], |row| {
            let permissions: String = row.get(1)?;
            Ok(PermissionEntry {
                app_name: row.get(0)?,
                requested_permissions: serde_json::from_str(&permissions).unwrap_or_default(),
                permission_state: match row.get::<_, String>(2)?.as_str() {
                    "AllowOnce" => PermissionState::AllowOnce,
                    "Always" => PermissionState::Always,
                    "Block" => PermissionState::Block,
                    _ => PermissionState::Block,
                },
            })
        })?;

        let entries: Result<Vec<_>, _> = rows.collect();
        Ok(entries?)
    }

    pub fn upsert(&mut self, entry: &PermissionEntry) -> Result<()> {
        let permissions = serde_json::to_string(&entry.requested_permissions)?;
        let state = match entry.permission_state {
            PermissionState::AllowOnce => "AllowOnce",
            PermissionState::Always => "Always",
            PermissionState::Block => "Block",
        };

        self.conn.execute(
            "INSERT OR REPLACE INTO permissions (app_name, requested_permissions, permission_state)
             VALUES (?1, ?2, ?3)",
            params![entry.app_name, permissions, state],
        )?;

        Ok(())
    }
}
