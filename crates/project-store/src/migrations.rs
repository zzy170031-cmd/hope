#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationScript {
    pub database_label: &'static str,
    pub version: u32,
    pub file_name: &'static str,
    pub contents: &'static str,
}

pub fn frozen_migrations() -> [MigrationScript; 2] {
    [
        MigrationScript {
            database_label: "hope-kb",
            version: 1,
            file_name: "migrations/hope-kb/0001_init.sql",
            contents: include_str!("../migrations/hope-kb/0001_init.sql"),
        },
        MigrationScript {
            database_label: "hope",
            version: 1,
            file_name: "migrations/hope/0001_init.sql",
            contents: include_str!("../migrations/hope/0001_init.sql"),
        },
    ]
}
