#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SheetContract {
    pub machine_name: &'static str,
    pub chinese_name: &'static str,
    pub columns: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportWorkbookContract {
    pub workbook_machine_name: &'static str,
    pub workbook_chinese_name: &'static str,
    pub sheets: &'static [SheetContract],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportFormatMapping {
    pub excel_headers: &'static [&'static str],
    pub json_keys: &'static [&'static str],
    pub markdown_headers: &'static [&'static str],
}

pub const CANONICAL_SHEETS: &[SheetContract] = &[
    SheetContract {
        machine_name: "project_meta",
        chinese_name: "项目元数据",
        columns: &["项目标识", "标题", "状态", "目标时长分钟", "更新时间戳"],
    },
    SheetContract {
        machine_name: "episode_meta",
        chinese_name: "集元数据",
        columns: &["集标识", "项目标识", "序号", "标题", "目标时长分钟"],
    },
    SheetContract {
        machine_name: "narrative_scene",
        chinese_name: "叙事场景",
        columns: &["叙事场景标识", "集标识", "序号", "标题", "内容摘要"],
    },
    SheetContract {
        machine_name: "render_segment",
        chinese_name: "RenderSegment",
        columns: &[
            "RenderSegment标识",
            "叙事场景标识",
            "序号",
            "起始镜头序号",
            "结束镜头序号",
            "目标时长秒",
            "实际时长秒",
        ],
    },
    SheetContract {
        machine_name: "cut",
        chinese_name: "Cut",
        columns: &[
            "Cut标识",
            "RenderSegment标识",
            "序号",
            "镜头描述",
            "对白",
            "时长秒",
        ],
    },
    SheetContract {
        machine_name: "prompt_package",
        chinese_name: "PromptPackage",
        columns: &["PromptPackage标识", "来源层级", "正文", "版本"],
    },
    SheetContract {
        machine_name: "handoff_zone",
        chinese_name: "交接带",
        columns: &[
            "HandoffZone标识",
            "RenderSegment标识",
            "起始边界",
            "结束边界",
            "边界类型",
        ],
    },
    SheetContract {
        machine_name: "hard_lock",
        chinese_name: "全局硬锁",
        columns: &["HardLock标识", "项目标识", "锁名", "锁值", "作用范围"],
    },
    SheetContract {
        machine_name: "stale_event",
        chinese_name: "Stale传播",
        columns: &[
            "StaleEvent标识",
            "来源层级",
            "来源标识",
            "目标层级",
            "目标标识",
            "追踪标识",
            "时间戳",
            "原因",
        ],
    },
    SheetContract {
        machine_name: "validation_report",
        chinese_name: "校验报告",
        columns: &["校验报告标识", "项目标识", "通过", "问题数", "更新时间戳"],
    },
    SheetContract {
        machine_name: "export_manifest",
        chinese_name: "导出清单",
        columns: &["导出清单标识", "项目标识", "工作簿版本", "状态"],
    },
    SheetContract {
        machine_name: "director_profile",
        chinese_name: "导演档案",
        columns: &["导演档案标识", "导演名称", "定位", "主风格", "镜头偏好"],
    },
    SheetContract {
        machine_name: "director_cut_sample",
        chinese_name: "导演切样",
        columns: &[
            "导演切样标识",
            "导演档案标识",
            "样例标题",
            "样例内容",
            "代表性说明",
        ],
    },
    SheetContract {
        machine_name: "committee_template",
        chinese_name: "委员会模板",
        columns: &["模板标识", "模板名称", "适用场景", "成员构成", "职责描述"],
    },
    SheetContract {
        machine_name: "visual_term",
        chinese_name: "视觉术语",
        columns: &["术语标识", "中文术语", "类别", "定义", "别名"],
    },
    SheetContract {
        machine_name: "cinematography_term",
        chinese_name: "摄影术语",
        columns: &["术语标识", "中文术语", "类别", "定义", "别名"],
    },
    SheetContract {
        machine_name: "continuity_rule",
        chinese_name: "连续性规则",
        columns: &["规则标识", "规则名称", "规则说明", "适用层级"],
    },
];

pub const CANONICAL_WORKBOOK_CONTRACT: ExportWorkbookContract = ExportWorkbookContract {
    workbook_machine_name: "hope_export_workbook",
    workbook_chinese_name: "Hope 导出工作簿",
    sheets: CANONICAL_SHEETS,
};

pub const fn canonical_workbook_contract() -> ExportWorkbookContract {
    CANONICAL_WORKBOOK_CONTRACT
}

pub const fn export_format_mapping(sheet: &SheetContract) -> ExportFormatMapping {
    ExportFormatMapping {
        excel_headers: sheet.columns,
        json_keys: sheet.columns,
        markdown_headers: sheet.columns,
    }
}
