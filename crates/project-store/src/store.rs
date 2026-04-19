use crate::policy::DualSqliteConnectionPolicy;

#[derive(Debug, Clone)]
pub struct StoreSkeleton {
    pub connection_policy: DualSqliteConnectionPolicy,
}

impl StoreSkeleton {
    pub fn new(connection_policy: DualSqliteConnectionPolicy) -> Self {
        Self { connection_policy }
    }

    pub fn contract_summary(&self) -> &'static str {
        "hope-kb read-only, hope uses WAL + foreign_keys=ON"
    }
}
