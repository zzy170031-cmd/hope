import { describe, expect, it } from "vitest";
import { buildSanitizedKbSummary, KB_SNAPSHOT } from "./kb";

describe("kb snapshot", () => {
  it("contains the required canonical lists", async () => {
    expect(KB_SNAPSHOT.sceneTypes).toHaveLength(21);
    expect(KB_SNAPSHOT.allowedDurations).toEqual([5, 10, 15, 30, 45, 60]);

    const summary = await buildSanitizedKbSummary("urban_fantasy", 15);
    expect(summary.scene_type_count).toBe(21);
    expect(summary.scene_type_id).toBe("urban_fantasy");
    expect(summary.scene_type_label).toBe("都市奇幻");
    expect(summary.selected_sample_ids).toHaveLength(3);
    expect(summary.kb_snapshot_hash).toHaveLength(64);
  });
});
