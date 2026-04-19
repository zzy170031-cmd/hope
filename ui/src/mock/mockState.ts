import type {
  ExportValidationItem,
  PreviewItem,
  ProjectSummary,
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

export const MOCK_EXPORT_VALIDATION: ExportValidationItem[] = [
  {
    label: "Workbook 结构",
    value: "17 张 sheet",
    state: "待对齐",
  },
  {
    label: "中文字段",
    value: "统一采用 UTF-8",
    state: "正常",
  },
  {
    label: "Validation report",
    value: "等待真实 IPC 接入",
    state: "待补充",
  },
];
