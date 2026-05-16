export type TransportCapability = "browser_direct" | "proxy_required" | "unsupported";
export type ProviderStatus = "ready" | "missing_config" | "failed" | "unsupported";
export type OperationMode = "expand_story" | "rewrite_script" | "storyboard_from_body";

export interface ProviderDefinition {
  providerId: string;
  displayName: string;
  transportCapability: TransportCapability;
  defaultBaseUrl: string;
  endpoint: string;
  defaultModels: string[];
  statusHint: string;
}

export interface ProviderModelRecord {
  provider_id: string;
  model_id: string;
  display_name: string;
  base_url_host: string;
  endpoint: string;
  transport_capability: TransportCapability;
  cors_check_result: string;
  last_connection_test: string | null;
  status: ProviderStatus;
}

export interface ProviderFormState {
  providerId: string;
  baseUrl: string;
  endpoint: string;
  apiKey: string;
  model: string;
  writingModel: string;
  directorModel: string;
  validatorModel: string;
  persistApiKey: boolean;
  connectionStatus: ProviderStatus;
  connectionMessage: string;
  corsCheckResult: string;
  lastConnectionTest: string | null;
}

export interface SourceFacts {
  characters: string[];
  locations: string[];
  events: string[];
  constraints: string[];
  visibleObjects: string[];
  beats: string[];
}

export interface RulePack {
  id: string;
  name: string;
  intent: string;
  influenceAxes: string[];
  directives: string[];
  negativeConstraints: string[];
}

export interface DurationCapacityProfile {
  duration: number;
  targetShotCount: number;
  beatDensity: string;
  pacingDirective: string;
}

export interface StoryboardTask {
  task_id: string;
  task_hash: string;
  storyboard_task_hash: string;
  task_name: string;
  task_index: number;
  task_source_kind: "system_candidate" | "full_narrative" | "confirmed_body" | "custom_fragment";
  task_source_label: string;
  people_summary: string;
  script_fragment: string;
  original_fragment: string;
  queue_status: "pending" | "generated";
  current_body_source: "accepted_body";
  accepted_body_hash: string;
  scene_type_id: string;
  scene_type_label: string;
  target_duration_seconds: number;
  kb_snapshot_hash: string;
  selected_sample_ids: string[];
  selected_kb_rules: string[];
  writing_group_rule_pack_ids: string[];
  director_group_rule_pack_ids: string[];
  output_intent: "storyboard_rows";
  source_lineage: "accepted_body";
  generation_group: string;
  estimated_shot_count: number;
  duration_allocation_strategy: string;
  continuity_status: string;
  created_from_confirmed_body: boolean;
  preserve_previous_task: boolean;
  rows_hash: string;
  confirmed_row_hash: string;
  stale_state: boolean;
  sync_status: "fresh" | "stale";
  stale_reasons: string[];
  created_at: string;
}

export interface KbSnapshot {
  snapshotVersion: string;
  sceneTypes: string[];
  allowedDurations: number[];
  writingRulePacks: RulePack[];
  directorRulePacks: RulePack[];
  sceneMappings: Record<
    string,
    {
      writingRulePackIds: string[];
      directorRulePackIds: string[];
      sceneProfile: string;
      negativeConstraints: string[];
      influenceAxes: string[];
    }
  >;
  durationProfiles: DurationCapacityProfile[];
}

export interface SanitizedKbSummary {
  kb_snapshot_hash: string;
  scene_type_count: number;
  scene_type_id: string;
  scene_type_label: string;
  duration_options: number[];
  selected_sample_ids: string[];
  selected_kb_rules: string[];
  writing_group_rule_pack_ids: string[];
  director_group_rule_pack_ids: string[];
  kb_context_summary: string;
  applied_to: Array<"narrative_body" | "storyboard_prompt">;
  influence_axes: string[];
  scene_profile: string;
  negative_constraints: string[];
  raw_kb_rows_included: 0;
  raw_sample_text_absent: true;
  source_register_absent: true;
  overlay_json_absent: true;
  prompt_body_absent: true;
}

export interface NarrativeResult {
  body: string;
  title: string;
  sourceFacts: SourceFacts;
  validatorResult: ValidationResult;
  evidence: GenerationEvidence;
}

export interface StoryboardRow {
  shot_index: number;
  person: string;
  shot_size: string;
  camera: string;
  visual_description: string;
  character_action: string;
  dialogue_or_narration: string;
  prompt_text: string;
  duration_seconds: number;
  status: string;
  note: string;
  is_user_edited?: boolean;
}

export interface StoryboardResult {
  rows: StoryboardRow[];
  validatorResult: ValidationResult;
  evidence: GenerationEvidence;
}

export interface ValidationIssue {
  code: string;
  message: string;
  severity: "info" | "warning" | "error";
}

export interface ValidationResult {
  passed: boolean;
  summary: string;
  issues: ValidationIssue[];
}

export interface GenerationEvidence {
  artifact_identity: string;
  source_lineage: string;
  source_lineage_evidence: string;
  source_hash: string;
  output_intent: "narrative_body" | "storyboard_rows";
  story_fact_frame: string;
  accepted_body_hash: string;
  storyboard_task: string;
  storyboard_task_hash: string;
  kb_snapshot_hash: string;
  selected_sample_ids: string[];
  selected_kb_rules: string[];
  writing_group_rule_pack_ids: string[];
  director_group_rule_pack_ids: string[];
  kb_context_summary: string;
  applied_to: Array<"narrative_body" | "storyboard_prompt">;
  influence_axes: string[];
  kb_oracle_affects_structure: boolean;
  raw_kb_rows_included: 0;
  raw_sample_text_absent: true;
  source_register_absent: true;
  overlay_json_absent: true;
  prompt_body_absent: true;
  scene_type_id: string;
  scene_type_label: string;
  scene_type_canonical_coverage: number;
  scene_type_valid: boolean;
  target_duration_seconds: number;
  rows_hash: string;
  confirmed_row_hash: string;
  export_source_hash: string;
  prompt_compilation_version: string;
  warning_taxonomy: string[];
  warning_taxonomy_classified: boolean;
  hardfail_warning_absent: boolean;
  stale_state: boolean;
  stale_task_detected: boolean;
  stale_rows_detected: boolean;
  task_sync_status: "fresh" | "stale";
  rows_count: number;
  rows_match: boolean;
  prompt_text_present: boolean;
  prompt_text_boundary_passed: boolean;
  prompt_text_not_summary_only: boolean;
  visual_description_visible_frame_passed: boolean;
  visual_description_no_trace: boolean;
  prompt_compiled_after_final_row: boolean;
  person_field_valid: boolean;
  location_not_in_person: boolean;
  action_fragment_not_in_person: boolean;
  scene_term_not_in_person: boolean;
  generic_role_placeholder_absent: boolean;
  validator_pseudo_success_detected: boolean;
  fallback_used: boolean;
  provider_failover_used: boolean;
  local_candidate: boolean;
  row_prompt_visual_gate_evidence: string;
  field_aware_entity_gate_evidence: string;
  manual_edit_confirmed_row_evidence: string;
  blocked_warning_ui_evidence: string;
  human_review_spotcheck_evidence: string;
  export_evidence: string[];
  validator_result: ValidationResult;
  provider_id: string;
  model_id: string;
  base_url_host: string;
  cors_check_result: string;
  generated_at: string;
}

export interface GenerationContext {
  sourceText: string;
  sceneTypeId: string;
  sceneTypeLabel: string;
  targetDuration: number;
  mode: OperationMode;
  provider: ProviderFormState;
  kbSummary: SanitizedKbSummary;
}

export interface ModelResponse<T> {
  content: string;
  parsed: T;
  rawUsage?: unknown;
}

export interface ExportBundle {
  fileName: string;
  mimeType: string;
  content: string;
  kind: "storyboard" | "script";
  format: "excel";
}

export interface ExportAudit {
  file_name: string;
  format: "excel";
  kind: "storyboard" | "script";
  usable: boolean;
  contains_secret_leakage: boolean;
  contains_local_path: boolean;
  contains_raw_kb: boolean;
  sampled_export_preview_present: boolean;
  preview: string[];
}

export interface QaTrace {
  provider_id: string;
  model_id: string;
  base_url_host: string;
  connection_status: ProviderStatus;
  connection_message: string;
  scene_type_id: string;
  scene_type_label: string;
  target_duration_seconds: number;
  accepted_body_ready: boolean;
  accepted_body_hash: string;
  storyboard_task_hash: string;
  storyboard_task_sync_status: "missing" | "fresh" | "stale";
  source_lineage: string;
  kb_ready: boolean;
  kb_snapshot_hash: string;
  selected_sample_ids: string[];
  selected_kb_rules: string[];
  writing_group_rule_pack_ids: string[];
  director_group_rule_pack_ids: string[];
  kb_context_summary: string;
  applied_to: Array<"narrative_body" | "storyboard_prompt">;
  influence_axes: string[];
  kb_oracle_affects_structure: boolean;
  raw_kb_rows_included: 0;
  raw_sample_text_absent: true;
  source_register_absent: true;
  overlay_json_absent: true;
  prompt_body_absent: true;
  rows_count: number;
  rows_match: boolean;
  prompt_text_present: boolean;
  prompt_text_boundary_passed: boolean;
  prompt_text_not_summary_only: boolean;
  visual_description_visible_frame_passed: boolean;
  visual_description_no_trace: boolean;
  prompt_compiled_after_final_row: boolean;
  stale_task_detected: boolean;
  stale_rows_detected: boolean;
  person_field_valid: boolean;
  location_not_in_person: boolean;
  action_fragment_not_in_person: boolean;
  scene_term_not_in_person: boolean;
  generic_role_placeholder_absent: boolean;
  validator_pseudo_success_detected: boolean;
  warning_taxonomy_classified: boolean;
  warning_taxonomy: string[];
  hardfail_warning_absent: boolean;
  fallback_used: boolean;
  provider_failover_used: boolean;
  local_candidate: boolean;
  export_audit: ExportAudit[];
}
