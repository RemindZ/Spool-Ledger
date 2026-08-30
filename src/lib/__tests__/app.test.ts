// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  discover: vi.fn(),
  catalogSources: vi.fn(),
  catalogTargets: vi.fn(),
  chooseManualSourceFolder: vi.fn(),
  removeManualSourceFolder: vi.fn(),
  previewNames: vi.fn(),
  buildPlan: vi.fn(),
  resolvePlanDependencies: vi.fn(),
  printerArtwork: vi.fn(),
  executePlan: vi.fn(),
  synchronizeRun: vi.fn(),
  cancelRun: vi.fn(),
  recordAmsVerification: vi.fn(),
  restorePreview: vi.fn(),
  restoreOwned: vi.fn(),
}));

vi.mock("../api", () => ({ api: mocks }));

import App from "../../App.svelte";
import { createInitialState, persistWorkspacePreferences } from "../state";

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
  artwork_available: false,
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
  source_precondition_fingerprint: "source-hash",
  material_settings_fingerprint: "material-hash",
  precondition_fingerprint: null,
  identity_fingerprint: {
    name: "polymaker pla panchroma satin",
    printer: "bambu lab h2c 0.4 nozzle",
    nozzle: "0.4",
  },
  action: "create" as const,
  conflict: null,
};

beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function showModal() {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function close() {
    this.removeAttribute("open");
  };
  localStorage.clear();
  const workspace = createInitialState();
  workspace.setupComplete = true;
  workspace.enabledPrinterIds = new Set([printer.id]);
  persistWorkspacePreferences(localStorage, workspace);
  vi.clearAllMocks();
  mocks.discover.mockResolvedValue({
    source_root_ids: ["orca-system"],
    manual_source_roots: [],
    target_catalog_ids: ["bambu-system"],
    orca_slicer_detected: true,
    bambu_studio_detected: true,
    platform: "windows",
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
  mocks.chooseManualSourceFolder.mockResolvedValue(null);
  mocks.removeManualSourceFolder.mockResolvedValue({
    source_root_ids: ["orca-system"],
    manual_source_roots: [],
    target_catalog_ids: ["bambu-system"],
    orca_slicer_detected: true,
    bambu_studio_detected: true,
    platform: "windows",
    accounts: [
      {
        id: "account-ready",
        eligibility: "eligible",
        filament_profile_count: 0,
      },
    ],
  });
  mocks.previewNames.mockResolvedValue([
    {
      preset_before: operation.preset_name,
      preset_name: operation.preset_name,
      ams_before: operation.ams_name,
      ams_name: operation.ams_name,
    },
  ]);
  mocks.buildPlan.mockResolvedValue({
    status: "ready",
    plan: { id: "plan-1", operations: [operation] },
  });
  mocks.resolvePlanDependencies.mockResolvedValue({
    status: "ready",
    plan: { id: "plan-1", operations: [operation] },
  });
  mocks.executePlan.mockResolvedValue({
    run_id: "run-1",
    plan_id: "plan-1",
    committed_files: 3,
    created_files: 3,
    updated_files: 0,
    deleted_files: 0,
    skipped_operations: 0,
    backup_sha256: "backup-sha256",
    backup_file_count: 3,
    receipt_path: "receipt.json",
  });
  mocks.synchronizeRun.mockImplementation(
    async (
      runId: string,
      onProgress: (event: { run_id: string; phase: string }) => void,
    ) => {
      onProgress({ run_id: runId, phase: "launching" });
      onProgress({ run_id: runId, phase: "monitoring" });
      return {
        timed_out: false,
        highest_evidence: "cloud_id_assigned",
        observations: [
          {
            operation_id: operation.id,
            info_path: "profile.info",
            evidence: "cloud_id_assigned",
            state: "cloud_id_assigned",
            setting_id: "PFUSunique123",
            diagnostic: null,
          },
        ],
      };
    },
  );
  mocks.cancelRun.mockResolvedValue(undefined);
});

afterEach(cleanup);

describe("application workflow", () => {
  it("shows first-run defaults once before entering the migration workspace", async () => {
    localStorage.clear();
    render(App);

    expect(
      await screen.findByRole("heading", { name: "Set your migration route" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "Prepare migration" }),
    ).not.toBeInTheDocument();

    await fireEvent.click(screen.getByLabelText("Enable Bambu Lab H2C"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Continue to migration" }),
    );

    expect(
      await screen.findByRole("heading", { name: "Prepare migration" }),
    ).toBeInTheDocument();
    expect(JSON.parse(localStorage.getItem("bfm.workspace")!)).toMatchObject({
      version: 2,
      setup_complete: true,
      enabled_printer_ids: ["official:H2C"],
      selected_nozzles: { "official:H2C": ["0.4"] },
    });
  });

  it("shows exact profile counts while source and target catalogs load", async () => {
    let finishSources!: (value: {
      catalog_id: string;
      sources: (typeof source)[];
    }) => void;
    let finishTargets!: (value: {
      catalog_id: string;
      printers: (typeof printer)[];
    }) => void;
    mocks.catalogSources.mockImplementation(
      (
        _rootIds: string[],
        onProgress: (event: {
          phase: string;
          processed: number;
          total: number;
        }) => void,
      ) => {
        onProgress({ phase: "loading", processed: 42, total: 100 });
        return new Promise((resolve) => {
          finishSources = resolve;
        });
      },
    );
    mocks.catalogTargets.mockImplementation(
      (
        _catalogId: string,
        onProgress: (event: {
          phase: string;
          processed: number;
          total: number;
        }) => void,
      ) => {
        onProgress({ phase: "resolving", processed: 12, total: 30 });
        return new Promise((resolve) => {
          finishTargets = resolve;
        });
      },
    );

    render(App);

    expect(await screen.findByText("42 / 100 profiles")).toBeInTheDocument();
    expect(
      screen.getByRole("progressbar", { name: "Profile loading progress" }),
    ).toHaveAttribute("aria-valuenow", "42");
    finishSources({ catalog_id: "sources-1", sources: [source] });

    expect(await screen.findByText("12 / 30 profiles")).toBeInTheDocument();
    expect(
      screen.getByText("Indexing Bambu target templates"),
    ).toBeInTheDocument();
    finishTargets({ catalog_id: "targets-1", printers: [printer] });

    expect(await screen.findByText(source.name)).toBeInTheDocument();
  });

  it("orders the real setup route and reports derived setup counts", async () => {
    render(App);
    await screen.findByText(source.name);

    expect(
      screen.getByRole("heading", { name: "Prepare migration" }),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Filament migration route"),
    ).toBeInTheDocument();
    const sourceHeading = screen.getByRole("heading", {
      name: "Filament profiles",
    });
    const namingHeading = screen.getByRole("heading", { name: "Naming" });
    const targetHeading = screen.getByRole("heading", {
      name: "Printers & nozzles",
    });
    expect(
      sourceHeading.compareDocumentPosition(namingHeading) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      namingHeading.compareDocumentPosition(targetHeading) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();

    await fireEvent.click(screen.getByLabelText(`Select ${source.name}`));
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    expect(screen.getByLabelText("1 source")).toBeInTheDocument();
    expect(screen.getByLabelText("1 nozzle")).toBeInTheDocument();
    expect(screen.getByLabelText("2 outputs")).toBeInTheDocument();
  });

  it("navigates four truthful workspace views with the keyboard", async () => {
    render(App);
    await screen.findByText(source.name);

    const setup = screen.getByRole("tab", { name: "Setup" });
    expect(setup).toHaveAttribute("aria-selected", "true");

    await fireEvent.keyDown(setup, { key: "ArrowRight" });
    const review = screen.getByRole("tab", { name: /Review plan/ });
    expect(review).toHaveAttribute("aria-selected", "true");
    expect(
      screen.getByText(
        "Build a migration plan to review exact output operations.",
      ),
    ).toBeInTheDocument();

    await fireEvent.keyDown(review, { key: "End" });
    expect(screen.getByRole("tab", { name: "Restore" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(
      screen.getByText(
        "Commit a migration before previewing journal-owned restore paths.",
      ),
    ).toBeInTheDocument();
  });

  it("discovers catalogs, builds an explicit plan, and commits only after review", async () => {
    render(App);

    expect(await screen.findByText(source.name)).toBeInTheDocument();
    expect(mocks.catalogSources).toHaveBeenCalledWith(
      ["orca-system"],
      expect.any(Function),
    );
    expect(mocks.catalogTargets).toHaveBeenCalledWith(
      "bambu-system",
      expect.any(Function),
    );
    expect(screen.queryByText("Show custom printers")).not.toBeInTheDocument();

    await fireEvent.click(screen.getByLabelText(`Select ${source.name}`));
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Build migration plan" }),
    );

    expect(
      await screen.findByDisplayValue("Panchroma Satin - H2C"),
    ).toBeInTheDocument();
    expect(mocks.buildPlan).toHaveBeenCalledWith(
      expect.objectContaining({
        source_ids: ["orca:satin"],
        nozzles: [{ printer_id: "official:H2C", diameters: ["0.4"] }],
        destination_account_id: "account-ready",
        naming: {
          preset_rules: [],
          ams_rules: [],
          overrides: [],
          conflict_decisions: [],
        },
      }),
    );

    expect(mocks.executePlan).not.toHaveBeenCalled();
    await fireEvent.click(
      screen.getByRole("button", { name: "Commit migration" }),
    );
    const confirmation = screen.getByRole("dialog", {
      name: "Commit plan plan-1?",
    });
    expect(mocks.executePlan).not.toHaveBeenCalled();
    expect(
      within(confirmation).getByRole("button", { name: "Cancel" }),
    ).toHaveFocus();
    expect(confirmation).toHaveTextContent(
      "Cloud persistence and AMS visibility are not guaranteed.",
    );
    await fireEvent.click(
      within(confirmation).getByRole("button", { name: "Commit migration" }),
    );
    expect(
      await screen.findByText("3 local files committed"),
    ).toBeInTheDocument();
    expect(mocks.executePlan).toHaveBeenCalledWith(
      "plan-1",
      expect.any(Function),
    );
    await waitFor(() =>
      expect(mocks.synchronizeRun).toHaveBeenCalledWith(
        "run-1",
        expect.any(Function),
      ),
    );
    expect(await screen.findByText("Cloud ID assigned")).toBeInTheDocument();
  });

  it("manages enabled destination printers from the cogwheel", async () => {
    render(App);
    await screen.findByText(source.name);

    await fireEvent.click(
      screen.getByRole("button", { name: "Manage enabled printers" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage enabled printers",
    });
    await fireEvent.click(
      within(dialog).getByLabelText("Enable Bambu Lab H2C"),
    );
    await fireEvent.click(
      within(dialog).getByRole("button", { name: "Save changes" }),
    );

    expect(
      screen.getByText(
        "No printers enabled. Use the settings button to add an official Bambu printer.",
      ),
    ).toBeInTheDocument();
    expect(JSON.parse(localStorage.getItem("bfm.workspace")!)).toMatchObject({
      enabled_printer_ids: [],
      selected_nozzles: {},
    });
  });

  it("resolves a missing target through the typed dependency dialog before creating a plan", async () => {
    const issue = {
      id: "target-template:pet-cf:h2c:0.4",
      expected_name: "Generic PET-CF @BBL H2C",
      material: "PET-CF",
      printer_id: printer.id,
      printer_name: printer.name,
      nozzle: "0.4",
      affected_sources: [{ id: source.id, name: source.name }],
      installed_candidates: [
        { profile_name: "Bambu PET-CF @BBL H2C", recommended: true },
      ],
      source_candidates: [],
      diagnostic: "The expected generic target is not installed.",
    };
    mocks.buildPlan.mockResolvedValueOnce({
      status: "needs_resolution",
      issues: [issue],
    });

    render(App);
    await screen.findByText(source.name);
    await fireEvent.click(screen.getByLabelText(`Select ${source.name}`));
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Build migration plan" }),
    );

    const dialog = await screen.findByRole("dialog", {
      name: "Target profile required",
    });
    expect(mocks.executePlan).not.toHaveBeenCalled();
    await fireEvent.click(
      within(dialog).getByRole("radio", {
        name: /Use Bambu PET-CF @BBL H2C for Bambu Lab H2C/,
      }),
    );
    await fireEvent.click(
      within(dialog).getByRole("button", { name: "Resolve dependencies" }),
    );

    await waitFor(() =>
      expect(mocks.resolvePlanDependencies).toHaveBeenCalledWith(
        expect.objectContaining({
          source_ids: [source.id],
          target_catalog_id: "targets-1",
        }),
        [
          {
            issue_id: issue.id,
            action: "use_installed",
            profile_name: "Bambu PET-CF @BBL H2C",
            source_id: null,
          },
        ],
      ),
    );
    expect(
      await screen.findByDisplayValue("Panchroma Satin - H2C"),
    ).toBeInTheDocument();
  });

  it("does not launch synchronization for an all-skip no-op plan", async () => {
    mocks.buildPlan.mockResolvedValue({
      status: "ready",
      plan: { id: "plan-skip", operations: [{ ...operation, action: "skip" }] },
    });
    mocks.executePlan.mockResolvedValue({
      run_id: "run-skip",
      plan_id: "plan-skip",
      committed_files: 0,
      created_files: 0,
      updated_files: 0,
      deleted_files: 0,
      skipped_operations: 1,
      backup_sha256: "backup-sha256",
      backup_file_count: 0,
      receipt_path: "receipt.json",
    });
    render(App);
    await screen.findByText(source.name);
    await fireEvent.click(screen.getByLabelText(`Select ${source.name}`));
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Build migration plan" }),
    );
    await screen.findByText("Skip");
    await fireEvent.click(
      screen.getByRole("button", { name: "Commit migration" }),
    );
    await fireEvent.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Commit migration",
      }),
    );

    expect(
      await screen.findByText("0 local files committed"),
    ).toBeInTheDocument();
    expect(mocks.synchronizeRun).not.toHaveBeenCalled();
  });

  it("adds and removes a native-picked manual source without passing a path", async () => {
    const manual = {
      id: "source:manual:bambu:1234",
      path: "C:\\Profiles\\Bambu\\filament",
      source_app: "bambu_studio" as const,
      source_kind: "user_custom" as const,
      active: true,
      error: null,
    };
    mocks.chooseManualSourceFolder.mockResolvedValue({
      source_root_ids: ["orca-system", manual.id],
      manual_source_roots: [manual],
      target_catalog_ids: ["bambu-system"],
      accounts: [
        {
          id: "account-ready",
          eligibility: "eligible",
          filament_profile_count: 0,
        },
      ],
    });
    render(App);
    await screen.findByText(source.name);

    await fireEvent.click(screen.getByText("Source folders"));
    await fireEvent.change(screen.getByLabelText("Source application"), {
      target: { value: "bambu_studio" },
    });
    await fireEvent.change(screen.getByLabelText("Source kind"), {
      target: { value: "user_custom" },
    });
    await fireEvent.click(
      screen.getByRole("button", { name: "Choose source folder" }),
    );

    await waitFor(() =>
      expect(mocks.chooseManualSourceFolder).toHaveBeenCalledWith({
        source_app: "bambu_studio",
        source_kind: "user_custom",
      }),
    );
    expect(mocks.catalogSources).toHaveBeenLastCalledWith(
      ["orca-system", manual.id],
      expect.any(Function),
    );
    expect(await screen.findByText(manual.path)).toBeInTheDocument();

    await fireEvent.click(
      screen.getByRole("button", {
        name: `Remove manual source ${manual.path}`,
      }),
    );
    await waitFor(() =>
      expect(mocks.removeManualSourceFolder).toHaveBeenCalledWith(manual.id),
    );
    expect(screen.queryByText(manual.path)).not.toBeInTheDocument();
  });

  it("rebuilds a blocked plan with an explicit conflict decision", async () => {
    const blocked = {
      ...operation,
      action: "block" as const,
      conflict: {
        kind: "settings_mismatch" as const,
        message: "Existing settings differ",
      },
    };
    const updated = { ...operation, action: "update" as const, conflict: null };
    mocks.buildPlan
      .mockResolvedValueOnce({
        status: "ready",
        plan: { id: "plan-blocked", operations: [blocked] },
      })
      .mockResolvedValueOnce({
        status: "ready",
        plan: { id: "plan-updated", operations: [updated] },
      });
    render(App);
    await screen.findByText(source.name);
    await fireEvent.click(screen.getByLabelText(`Select ${source.name}`));
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Build migration plan" }),
    );
    await screen.findByText("Existing settings differ");

    await fireEvent.change(
      screen.getByLabelText(`Conflict decision for ${source.name}`),
      { target: { value: "update" } },
    );

    await waitFor(() => expect(mocks.buildPlan).toHaveBeenCalledTimes(2));
    expect(mocks.buildPlan).toHaveBeenLastCalledWith(
      expect.objectContaining({
        naming: expect.objectContaining({
          conflict_decisions: [
            {
              source_id: source.id,
              printer_id: printer.id,
              nozzle: "0.4",
              choice: "update",
            },
          ],
        }),
      }),
    );
    expect(await screen.findByText("Update")).toBeInTheDocument();
  });

  it("rebuilds an existing plan when per-run outputs change", async () => {
    render(App);
    await screen.findByText(source.name);
    await fireEvent.click(screen.getByLabelText(`Select ${source.name}`));
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Build migration plan" }),
    );
    await screen.findByDisplayValue("Panchroma Satin - H2C");

    await fireEvent.click(screen.getByRole("tab", { name: "Setup" }));
    await fireEvent.click(screen.getByText("Advanced outputs"));
    await fireEvent.click(screen.getByLabelText("Create AMS custom filaments"));

    await waitFor(() => expect(mocks.buildPlan).toHaveBeenCalledTimes(2));
    expect(mocks.buildPlan).toHaveBeenLastCalledWith(
      expect.objectContaining({
        outputs: {
          slicing_presets: true,
          custom_filaments: false,
        },
      }),
    );
  });

  it("retains a timed-out local result and retries synchronization", async () => {
    mocks.synchronizeRun.mockImplementationOnce(async () => ({
      timed_out: true,
      highest_evidence: "created_local",
      observations: [
        {
          operation_id: operation.id,
          info_path: "profile.info",
          evidence: "created_local",
          state: "created_local_unsynchronized",
          setting_id: null,
          diagnostic: null,
        },
      ],
    }));
    render(App);
    await screen.findByText(source.name);
    await fireEvent.click(screen.getByLabelText(`Select ${source.name}`));
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Build migration plan" }),
    );
    await screen.findByDisplayValue("Panchroma Satin - H2C");
    await fireEvent.click(
      screen.getByRole("button", { name: "Commit migration" }),
    );
    await fireEvent.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Commit migration",
      }),
    );

    expect(
      await screen.findByText("Synchronization timed out"),
    ).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole("button", { name: "Retry synchronization" }),
    );

    await waitFor(() => expect(mocks.synchronizeRun).toHaveBeenCalledTimes(2));
    expect(await screen.findByText("Cloud ID assigned")).toBeInTheDocument();
  });

  it("cancels active monitoring by its committed run id", async () => {
    mocks.synchronizeRun.mockImplementationOnce(
      async (
        runId: string,
        onProgress: (event: { run_id: string; phase: string }) => void,
      ) => {
        onProgress({ run_id: runId, phase: "monitoring" });
        return new Promise(() => {});
      },
    );
    render(App);
    await screen.findByText(source.name);
    await fireEvent.click(screen.getByLabelText(`Select ${source.name}`));
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    await fireEvent.click(
      screen.getByRole("button", { name: "Build migration plan" }),
    );
    await screen.findByDisplayValue("Panchroma Satin - H2C");
    await fireEvent.click(
      screen.getByRole("button", { name: "Commit migration" }),
    );
    await fireEvent.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Commit migration",
      }),
    );

    await fireEvent.click(
      await screen.findByRole("button", { name: "Cancel monitoring" }),
    );

    expect(mocks.cancelRun).toHaveBeenCalledWith("run-1");
  });

  it("blocks planning for persisted slicing-only tokens in the AMS template", async () => {
    const workspace = createInitialState();
    workspace.selectedAccountId = "account-ready";
    workspace.setupComplete = true;
    workspace.enabledPrinterIds = new Set([printer.id]);
    workspace.selectedSourceIds = new Set([source.id]);
    workspace.selectedNozzles = { [printer.id]: new Set(["0.4"]) };
    workspace.amsTemplate = "{vendor} @{printer_code}";
    persistWorkspacePreferences(localStorage, workspace);

    render(App);
    await screen.findByText(source.name);

    expect(
      screen.getByText(
        "Remove slicing-only @, {printer_code} from this AMS template.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Build migration plan" }),
    ).toBeDisabled();
  });

  it("blocks planning until a source, nozzle, and eligible account are selected", async () => {
    render(App);
    await screen.findByText(source.name);
    const build = screen.getByRole("button", { name: "Build migration plan" });
    expect(build).toBeDisabled();
    expect(screen.getByText("Select a source and nozzle")).toBeInTheDocument();
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
