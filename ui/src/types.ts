export type ViewId = "projects" | "writer" | "preview" | "export";

export type LoadStatus = "loading" | "ready" | "empty" | "error";

export type ExportValidationState = "正常" | "待补充" | "待对齐";

export interface ProjectSummary {
  id: string;
  name: string;
  status: string;
  updatedAt: string;
  episodeCount: number;
}

export interface WriterLayerSnapshot {
  synopsis: string;
  story: string;
  screenplay: string;
  storyboard: string;
}

export interface PreviewItem {
  id: string;
  label: string;
  duration: string;
  note: string;
}

export interface ExportValidationItem {
  label: string;
  value: string;
  state: ExportValidationState;
}

export interface ProjectCreateOrSwitchRequest {
  project_id?: string;
  project_name?: string;
}

export interface WriterEntrySnapshotRequest {
  project_id: string;
}

export interface StoryboardRenderSegmentCutPreviewSnapshotRequest {
  project_id: string;
  episode_id?: string;
  narrative_scene_id?: string;
  render_segment_id?: string;
  scene_type?: string;
}

export interface ValidationExportPanelSnapshotRequest {
  project_id: string;
}
