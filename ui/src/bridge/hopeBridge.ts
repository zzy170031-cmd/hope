import type {
  ExportValidationItem,
  PreviewItem,
  ProjectCreateOrSwitchRequest,
  ProjectSummary,
  StoryboardRenderSegmentCutPreviewSnapshotRequest,
  ValidationExportPanelSnapshotRequest,
  WriterEntrySnapshotRequest,
  WriterLayerSnapshot,
} from "../types";
import {
  MOCK_EXPORT_VALIDATION,
  MOCK_PREVIEW_ITEMS,
  MOCK_PROJECTS,
  MOCK_WRITER_SNAPSHOT,
} from "../mock/mockState";

export const HOPE_TAURI_COMMANDS = {
  projectCreateOrSwitch: "project_create_or_switch",
  writerEntrySnapshot: "writer_entry_snapshot",
  storyboardRenderSegmentCutPreviewSnapshot:
    "storyboard_rendersegment_cut_preview_snapshot",
  validationExportPanelSnapshot: "validation_export_panel_snapshot",
} as const;

type HopeCommandName = (typeof HOPE_TAURI_COMMANDS)[keyof typeof HOPE_TAURI_COMMANDS];

type HopeCommandPayload =
  | ProjectCreateOrSwitchRequest
  | WriterEntrySnapshotRequest
  | StoryboardRenderSegmentCutPreviewSnapshotRequest
  | ValidationExportPanelSnapshotRequest
  | undefined;

function delay(ms: number) {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });
}

export async function invokeHopeCommand<T>(
  command: HopeCommandName,
  _payload?: HopeCommandPayload,
): Promise<T> {
  await delay(200);

  switch (command) {
    case HOPE_TAURI_COMMANDS.projectCreateOrSwitch:
      return MOCK_PROJECTS as T;
    case HOPE_TAURI_COMMANDS.writerEntrySnapshot:
      return MOCK_WRITER_SNAPSHOT as T;
    case HOPE_TAURI_COMMANDS.storyboardRenderSegmentCutPreviewSnapshot:
      return MOCK_PREVIEW_ITEMS as T;
    case HOPE_TAURI_COMMANDS.validationExportPanelSnapshot:
      return MOCK_EXPORT_VALIDATION as T;
    default:
      throw new Error(`Unmapped Hope UI command: ${command}`);
  }
}

export async function loadProjectList() {
  return invokeHopeCommand<ProjectSummary[]>(
    HOPE_TAURI_COMMANDS.projectCreateOrSwitch,
  );
}

export async function loadWriterSnapshot(project_id = "project-week3-001") {
  return invokeHopeCommand<WriterLayerSnapshot>(
    HOPE_TAURI_COMMANDS.writerEntrySnapshot,
    { project_id },
  );
}

export async function loadPreviewSnapshot(project_id = "project-week3-001") {
  return invokeHopeCommand<Record<"storyboard" | "renderSegment" | "cuts", PreviewItem[]>>(
    HOPE_TAURI_COMMANDS.storyboardRenderSegmentCutPreviewSnapshot,
    { project_id },
  );
}

export async function loadExportValidationSnapshot(project_id = "project-week3-001") {
  return invokeHopeCommand<ExportValidationItem[]>(
    HOPE_TAURI_COMMANDS.validationExportPanelSnapshot,
    { project_id },
  );
}
