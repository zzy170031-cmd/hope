PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS schema_meta (
    schema_name TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS project (
    project_id TEXT PRIMARY KEY,
    标题 TEXT NOT NULL,
    状态 TEXT NOT NULL,
    目标时长分钟 INTEGER NOT NULL,
    创建时间戳 INTEGER NOT NULL,
    更新时间戳 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS episode (
    episode_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES project(project_id) ON DELETE CASCADE,
    序号 INTEGER NOT NULL,
    标题 TEXT NOT NULL,
    目标时长分钟 INTEGER NOT NULL,
    状态 TEXT NOT NULL,
    创建时间戳 INTEGER NOT NULL,
    更新时间戳 INTEGER NOT NULL,
    UNIQUE(project_id, 序号)
);

CREATE TABLE IF NOT EXISTS narrative_scene (
    narrative_scene_id TEXT PRIMARY KEY,
    episode_id TEXT NOT NULL REFERENCES episode(episode_id) ON DELETE CASCADE,
    序号 INTEGER NOT NULL,
    标题 TEXT NOT NULL,
    内容摘要 TEXT NOT NULL,
    创建时间戳 INTEGER NOT NULL,
    更新时间戳 INTEGER NOT NULL,
    UNIQUE(episode_id, 序号)
);

CREATE TABLE IF NOT EXISTS render_segment (
    render_segment_id TEXT PRIMARY KEY,
    narrative_scene_id TEXT NOT NULL REFERENCES narrative_scene(narrative_scene_id) ON DELETE CASCADE,
    序号 INTEGER NOT NULL,
    起始镜头序号 INTEGER NOT NULL,
    结束镜头序号 INTEGER NOT NULL,
    目标时长秒 INTEGER NOT NULL,
    实际时长秒 INTEGER,
    状态 TEXT NOT NULL,
    创建时间戳 INTEGER NOT NULL,
    更新时间戳 INTEGER NOT NULL,
    UNIQUE(narrative_scene_id, 序号)
);

CREATE TABLE IF NOT EXISTS cut (
    cut_id TEXT PRIMARY KEY,
    render_segment_id TEXT NOT NULL REFERENCES render_segment(render_segment_id) ON DELETE CASCADE,
    序号 INTEGER NOT NULL,
    镜头描述 TEXT NOT NULL,
    对白 TEXT NOT NULL,
    时长秒 INTEGER NOT NULL,
    状态 TEXT NOT NULL,
    创建时间戳 INTEGER NOT NULL,
    更新时间戳 INTEGER NOT NULL,
    UNIQUE(render_segment_id, 序号)
);

CREATE TABLE IF NOT EXISTS handoff_zone (
    handoff_zone_id TEXT PRIMARY KEY,
    render_segment_id TEXT NOT NULL REFERENCES render_segment(render_segment_id) ON DELETE CASCADE,
    起始边界 TEXT NOT NULL,
    结束边界 TEXT NOT NULL,
    边界类型 TEXT NOT NULL,
    备注 TEXT NOT NULL DEFAULT '',
    创建时间戳 INTEGER NOT NULL,
    更新时间戳 INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS hard_lock (
    hard_lock_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES project(project_id) ON DELETE CASCADE,
    锁名 TEXT NOT NULL,
    锁值 TEXT NOT NULL,
    作用范围 TEXT NOT NULL DEFAULT 'project',
    更新时间戳 INTEGER NOT NULL,
    UNIQUE(project_id, 锁名)
);

CREATE TABLE IF NOT EXISTS stale_propagation_event (
    stale_event_id TEXT PRIMARY KEY,
    来源层级 TEXT NOT NULL,
    来源标识 TEXT NOT NULL,
    目标层级 TEXT NOT NULL,
    目标标识 TEXT NOT NULL,
    追踪标识 TEXT NOT NULL,
    时间戳 INTEGER NOT NULL,
    原因 TEXT NOT NULL,
    UNIQUE(追踪标识, 来源层级, 来源标识, 目标层级, 目标标识)
);
