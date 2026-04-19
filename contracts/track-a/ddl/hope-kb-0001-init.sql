PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_meta (
    schema_name TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS kb_snapshot (
    snapshot_id TEXT PRIMARY KEY,
    snapshot_hash TEXT NOT NULL,
    seed_format TEXT NOT NULL,
    source_name TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS director_profile (
    director_profile_id TEXT PRIMARY KEY,
    导演名 TEXT NOT NULL,
    定位 TEXT NOT NULL,
    主风格 TEXT NOT NULL,
    镜头偏好 TEXT NOT NULL,
    连续性偏好 TEXT NOT NULL,
    更新时间戳 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS director_cut_sample (
    director_cut_sample_id TEXT PRIMARY KEY,
    director_profile_id TEXT NOT NULL REFERENCES director_profile(director_profile_id) ON DELETE CASCADE,
    样例标题 TEXT NOT NULL,
    样例内容 TEXT NOT NULL,
    代表性说明 TEXT NOT NULL,
    更新时间戳 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS committee_template (
    template_id TEXT PRIMARY KEY,
    模板名 TEXT NOT NULL,
    适用场景 TEXT NOT NULL,
    成员构成 TEXT NOT NULL,
    职责描述 TEXT NOT NULL,
    更新时间戳 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS visual_term (
    term_id TEXT PRIMARY KEY,
    中文术语 TEXT NOT NULL,
    类别 TEXT NOT NULL,
    定义 TEXT NOT NULL,
    别名 TEXT NOT NULL DEFAULT '',
    更新时间戳 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS cinematography_term (
    term_id TEXT PRIMARY KEY,
    中文术语 TEXT NOT NULL,
    类别 TEXT NOT NULL,
    定义 TEXT NOT NULL,
    别名 TEXT NOT NULL DEFAULT '',
    更新时间戳 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS continuity_rule (
    rule_id TEXT PRIMARY KEY,
    规则名 TEXT NOT NULL,
    规则说明 TEXT NOT NULL,
    适用层级 TEXT NOT NULL,
    更新时间戳 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS prompt_template (
    template_id TEXT PRIMARY KEY,
    模板名 TEXT NOT NULL,
    适用层级 TEXT NOT NULL,
    模板正文 TEXT NOT NULL,
    输入字段清单 TEXT NOT NULL,
    更新时间戳 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS seed_import_batch (
    batch_id TEXT PRIMARY KEY,
    来源 TEXT NOT NULL,
    seed_format TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    imported_at INTEGER NOT NULL
);
