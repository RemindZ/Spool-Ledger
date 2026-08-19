// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  discover: vi.fn(),
  catalogSources: vi.fn(),
  catalogTargets: vi.fn(),
  previewNames: vi.fn(),
  buildPlan: vi.fn(),
  executePlan: vi.fn(),
  cancelRun: vi.fn(),
  recordAmsVerification: vi.fn(),
  restorePreview: vi.fn(),
  restoreOwned: vi.fn(),
}));

vi.mock("../api", () => ({ api: mocks }));

import App from "../../App.svelte";

const source = {
  id: "orca:satin",
  name: "Panchroma PLA Satin @BBL X1C",
  vendor: "Polymaker",
  material: "PLA",
  family: "Panchroma",
  variant: "Satin",
  source_app: "orca_slicer" as const,
  source_kind: "factory_system" as const,
  compatible_printers: ["Bambu Lab X1 Carbon"],
  migration_status: "new" as const,
  warnings: [],
};

const printer = {
  id: "official:H2C",
  name: "Bambu Lab H2C",
  code: "H2C",
  kind: "official" as const,
  verified: true,
  extruder_variants: ["Direct Drive Standard"],
  nozzles: [
    {
      id: "H2C:0.4",
      diameter: "0.4",
      printer_preset_name: "Bambu Lab H2C 0.4 nozzle",
      selected: false,
      supported: true,
    },
  ],
};

const operation = {
  id: "op-1",
  source_id: source.id,
  source_name: source.name,
  preset_name: "Panchroma Satin - H2C",
  ams_name: "Polymaker PLA Panchroma Satin",
  filament_id: "P1234567",
  printer_id: printer.id,
  printer_name: printer.name,
  printer_preset_name: "Bambu Lab H2C 0.4 nozzle",
  nozzle: "0.4",
  custom_unverified: false,
  source_settings_fingerprint: "source-hash",
  precondition_fingerprint: null,
  action: "create" as const,
  conflict: null,
};

beforeEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
  mocks.discover.mockResolvedValue({
    source_root_ids: ["orca-system"],
    target_catalog_ids: ["bambu-system"],
    accounts: [
      {
        id: "account-ready",
        eligibility: "eligible",
        filament_profile_count: 0,
      },
    ],
  });
  mocks.catalogSources.mockResolvedValue({
    catalog_id: "sources-1",
    sources: [source],
  });
  mocks.catalogTargets.mockResolvedValue({
    catalog_id: "targets-1",
    printers: [printer],
  });
  mocks.previewNames.mockResolvedValue([
    { preset_name: operation.preset_name, ams_name: operation.ams_name },
  ]);
  mocks.buildPlan.mockResolvedValue({
    plan: { id: "plan-1", operations: [operation] },
  });
  mocks.executePlan.mockResolvedValue({
    run_id: "run-1",
    plan_id: "plan-1",
    committed_files: 3,
    receipt_path: "receipt.json",
  });
});

afterEach(cleanup);

describe("application workflow", () => {
  it("discovers catalogs, builds an explicit plan, and commits only after review", async () => {
    render(App);

    expect(await screen.findByText(source.name)).toBeInTheDocument();
    expect(mocks.catalogSources).toHaveBeenCalledWith(["orca-system"]);
    expect(mocks.catalogTargets).toHaveBeenCalledWith("bambu-system", false);

    await fireEvent.click(screen.getByLabelText(`Select ${source.name}`));
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Build migration plan" }),
    );

    expect(
      await screen.findByText("Panchroma Satin - H2C"),
    ).toBeInTheDocument();
    expect(mocks.buildPlan).toHaveBeenCalledWith(
      expect.objectContaining({
        source_ids: ["orca:satin"],
        nozzles: [{ printer_id: "official:H2C", diameters: ["0.4"] }],
        destination_account_id: "account-ready",
      }),
    );

    expect(mocks.executePlan).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole("button", { name: "Commit migration" }),
    );
    expect(
      await screen.findByText("3 local files committed"),
    ).toBeInTheDocument();
    expect(mocks.executePlan).toHaveBeenCalledWith("plan-1");
  });

  it("blocks planning until a source, nozzle, and eligible account are selected", async () => {
    render(App);
    await screen.findByText(source.name);
    const build = screen.getByRole("button", { name: "Build migration plan" });
    expect(build).toBeDisabled();
    expect(
      screen.getByText("Select at least one source and one nozzle"),
    ).toBeInTheDocument();
  });

  it("surfaces discovery failures with a retry action", async () => {
    mocks.discover.mockRejectedValueOnce(
      new Error("Profile manifests could not be read"),
    );
    render(App);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Profile manifests could not be read",
    );
    await fireEvent.click(
      screen.getByRole("button", { name: "Retry discovery" }),
    );
    await waitFor(() => expect(mocks.discover).toHaveBeenCalledTimes(2));
    expect(await screen.findByText(source.name)).toBeInTheDocument();
  });
});
