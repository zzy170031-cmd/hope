import type {
  ExportValidationItem,
  PreviewItem,
  ProjectSummary,
  ValidationExportPanelSnapshot,
  WriterLayerSnapshot,
} from "../types";

export const MOCK_PROJECTS: ProjectSummary[] = [
  {
    id: "project-001",
    name: "晨雾计划",
    status: "进行中",
    updatedAt: "2026-04-19 18:20",
    episodeCount: 3,
  },
  {
    id: "project-002",
    name: "月面回声",
    status: "待整理",
    updatedAt: "2026-04-18 16:00",
    episodeCount: 1,
  },
];

export const MOCK_WRITER_SNAPSHOT: WriterLayerSnapshot = {
  synopsis: "一位失去短期记忆的导演助理，必须在午夜前找回断裂的任务线索。",
  story: "Story 层保留目标、冲突、时长约束的结构化骨架。",
  screenplay: "Screenplay 层展示分场景对白与动作说明的最小占位。",
  storyboard: "Storyboard 层等待 Track E 的共享 fixture 和真实导出链接入。",
};

export const MOCK_PREVIEW_ITEMS: Record<"storyboard" | "renderSegment" | "cuts", PreviewItem[]> = {
  storyboard: [
    {
      id: "sb-01",
      label: "镜头 01",
      duration: "16s",
      note: "共享 fixture 中的第一个 cut。",
    },
    {
      id: "sb-02",
      label: "镜头 02",
      duration: "16s",
      note: "共享 fixture 中的第二个 cut。",
    },
    {
      id: "sb-03",
      label: "镜头 03",
      duration: "16s",
      note: "共享 fixture 中的第三个 cut。",
    },
  ],
  renderSegment: [
    {
      id: "rs-01",
      label: "RenderSegment 01",
      duration: "48s",
      note: "共享 fixture 中唯一的 RenderSegment。",
    },
  ],
  cuts: [
    {
      id: "cut-01",
      label: "Cut 01",
      duration: "16s",
      note: "中景推入。",
    },
  ],
};

const MOCK_EXPORT_VALIDATION_SUMMARY: ExportValidationItem[] = [
  {
    label: "当前项目",
    value: "project-week3-001",
    state: "正常",
  },
  {
    label: "Validation 报告",
    value: "17 行检查 / 2 条阻塞",
    state: "阻塞",
  },
  {
    label: "修复建议",
    value: "2 条 KB-backed 建议",
    state: "正常",
  },
];

export const MOCK_EXPORT_VALIDATION_SNAPSHOT: ValidationExportPanelSnapshot = {
  projectId: "project-week3-001",
  summaryItems: MOCK_EXPORT_VALIDATION_SUMMARY,
  repairRecommendations: [
    {
      failureCode: "style_drift",
      failureName: "Style Drift",
      repairStrategy: "重新注入风格锁定语句，并在 render prompt 中保留 identity baseline。",
      repairPriority: "high",
      repairScope: "render_prompt_only",
      validatorHint: "Style Unity Validator",
      promptTemplateNames: ["Repair Style Lock"],
    },
    {
      failureCode: "chinese_prompt_noise",
      failureName: "Chinese Prompt Noise",
      repairStrategy: "将占位 prompt 改写为自然中文，去掉 TODO 和机器痕迹。",
      repairPriority: "medium",
      repairScope: "prompt_rendering_layer",
      validatorHint: "Prompt quality review",
      promptTemplateNames: ["Repair Prompt Language"],
    },
  ],
};
