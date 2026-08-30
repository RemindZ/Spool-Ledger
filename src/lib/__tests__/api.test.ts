import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke, openUrl } = vi.hoisted(() => ({
  invoke: vi.fn(),
  openUrl: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
  Channel: class {
    onmessage: ((event: unknown) => void) | null = null;
  },
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl }));

import { api } from "../api";
import type { BuildPlanRequest, TargetTemplateDecision } from "../types";

const request: BuildPlanRequest = {
  source_catalog_id: "sources:1",
  source_ids: ["profile:orca:PET-CF"],
  target_catalog_id: "targets:1",
  nozzles: [{ printer_id: "official:H2C", diameters: ["0.4"] }],
  destination_account_id: "account:1",
  preset_template: "{clean_name}",
  ams_template: "{clean_name}",
  outputs: { slicing_presets: true, custom_filaments: true },
  naming: {
    preset_rules: [],
    ams_rules: [],
    overrides: [],
    conflict_decisions: [],
  },
};

beforeEach(() => {
  invoke.mockReset();
  openUrl.mockReset();
});

describe("typed dependency and artwork IPC", () => {
  it("submits dependency decisions without changing the frozen request shape", async () => {
    const decisions: TargetTemplateDecision[] = [
      {
        issue_id: "target-template:pet-cf:h2c:0.4",
        action: "use_installed",
        profile_name: "Bambu PET-CF @BBL H2C",
        source_id: null,
      },
    ];
    invoke.mockResolvedValue({
      status: "ready",
      plan: { id: "plan:1", operations: [] },
    });

    await api.resolvePlanDependencies(request, decisions);

    expect(invoke).toHaveBeenCalledWith("resolve_plan_dependencies", {
      request: { request, decisions },
    });
  });

  it("opens only the fixed support URL in the system browser", async () => {
    openUrl.mockResolvedValue(undefined);

    await api.openSupportPage();

    expect(openUrl).toHaveBeenCalledOnce();
    expect(openUrl).toHaveBeenCalledWith("https://buymeacoffee.com/Remitec");
  });

  it("requests printer artwork only by opaque catalog and printer ids", async () => {
    const bytes = new Uint8Array([137, 80, 78, 71]);
    invoke.mockResolvedValue(bytes);

    await expect(api.printerArtwork("targets:1", "official:H2C")).resolves.toBe(
      bytes,
    );
    expect(invoke).toHaveBeenCalledWith("printer_artwork", {
      catalogId: "targets:1",
      printerId: "official:H2C",
    });
  });
});
