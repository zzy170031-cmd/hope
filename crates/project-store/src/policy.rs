use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqliteMode {
    ReadOnly,
    WalWrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqliteConnectionPolicy {
    pub database_label: &'static str,
    pub file_path: PathBuf,
    pub mode: SqliteMode,
    pub foreign_keys_enabled: bool,
}

impl SqliteConnectionPolicy {
    pub fn hope_kb(file_path: PathBuf) -> Self {
        Self {
            database_label: "hope-kb",
            file_path,
            mode: SqliteMode::ReadOnly,
            foreign_keys_enabled: true,
        }
    }

    pub fn hope(file_path: PathBuf) -> Self {
        Self {
            database_label: "hope",
            file_path,
            mode: SqliteMode::WalWrite,
            foreign_keys_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualSqliteConnectionPolicy {
    pub hope_kb: SqliteConnectionPolicy,
    pub hope: SqliteConnectionPolicy,
}

impl DualSqliteConnectionPolicy {
    pub fn new(hope_kb: PathBuf, hope: PathBuf) -> Self {
        Self {
            hope_kb: SqliteConnectionPolicy::hope_kb(hope_kb),
            hope: SqliteConnectionPolicy::hope(hope),
        }
    }
}
