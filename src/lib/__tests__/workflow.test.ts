// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import NamingPanel from "../components/NamingPanel.svelte";
import PlanTable from "../components/PlanTable.svelte";
import ResultSummary from "../components/ResultSummary.svelte";
import RunProgress from "../components/RunProgress.svelte";
import TargetPanel from "../components/TargetPanel.svelte";
import type { MigrationPlan, PrinterTarget } from "../types";

afterEach(cleanup);

const printers: PrinterTarget[] = [
  {
    id: "official:H2C",
    name: "Bambu Lab H2C",
    code: "H2C",
    kind: "official",
    verified: true,
    artwork_available: false,
    extruder_variants: ["Direct Drive Standard", "Direct Drive High Flow"],
    nozzles: [
      ...["0.2", "0.4"].map((diameter) => ({
        id: `H2C:${diameter}`,
        diameter,
        printer_preset_name: `Bambu Lab H2C ${diameter} nozzle`,
        selected: false,
        supported: true,
      })),
      {
        id: "H2C:1.0",
        diameter: "1.0",
        printer_preset_name: "Bambu Lab H2C 1.0 nozzle",
        selected: false,
        supported: false,
      },
    ],
  },
  {
    id: "custom:Workshop",
    name: "Workshop CoreXY",
    code: "WC",
    kind: "custom",
    verified: false,
    artwork_available: false,
    extruder_variants: ["Unknown custom target"],
    nozzles: [
      {
        id: "custom:0.4",
        diameter: "0.4",
        printer_preset_name: "Workshop CoreXY 0.4 nozzle",
        selected: false,
        supported: false,
      },
    ],
  },
];

const plan: MigrationPlan = {
  id: "plan-1",
  operations: [
    {
      id: "op-1",
      source_id: "source-1",
      source_name: "Panchroma PLA Satin",
      preset_name: "Panchroma Satin - H2C",
      ams_name: "Polymaker PLA Panchroma Satin",
      filament_id: "P1234567",
      printer_id: "official:H2C",
      printer_name: "Bambu Lab H2C",
      printer_preset_name: "Bambu Lab H2C 0.4 nozzle",
      nozzle: "0.4",
      custom_unverified: false,
      source_precondition_fingerprint: "source-a",
      material_settings_fingerprint: "material-a",
      precondition_fingerprint: null,
      identity_fingerprint: {
        name: "polymaker pla panchroma satin",
        printer: "bambu lab h2c 0.4 nozzle",
        nozzle: "0.4",
      },
      action: "block",
      conflict: {
        kind: "settings_mismatch",
        message: "Existing settings differ",
      },
    },
  ],
};

describe("workflow panels", () => {
  it("renders enabled official targets only and opens printer management", async () => {
    const onManage = vi.fn();
    render(TargetPanel, {
      props: {
        catalogId: "targets:1",
        printers,
        enabledPrinterIds: new Set(["official:H2C"]),
        selectedNozzles: {},
        onManage,
        onNozzleToggle: vi.fn(),
        onSelectAll: vi.fn(),
      },
    });

    expect(screen.getByText("Bambu Lab H2C")).toBeInTheDocument();
    expect(screen.queryByText("Workshop CoreXY")).not.toBeInTheDocument();
    expect(screen.getByLabelText("1.0 mm")).toBeDisabled();
    expect(
      screen.getByText("Unavailable for this official printer"),
    ).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole("button", { name: "Manage enabled printers" }),
    );
    expect(onManage).toHaveBeenCalledOnce();
  });

  it("selects nozzles independently and exposes explicit select-all", async () => {
    const onNozzleToggle = vi.fn();
    const onSelectAll = vi.fn();
    render(TargetPanel, {
      props: {
        catalogId: "targets:1",
        printers,
        enabledPrinterIds: new Set(["official:H2C"]),
        selectedNozzles: {},
        onManage: vi.fn(),
        onNozzleToggle,
        onSelectAll,
      },
    });
    await fireEvent.click(screen.getByLabelText("0.4 mm"));
    expect(onNozzleToggle).toHaveBeenCalledWith("official:H2C", "0.4");
    await fireEvent.click(
      screen.getByRole("button", {
        name: "Select all nozzles for Bambu Lab H2C",
      }),
    );
    expect(onSelectAll).toHaveBeenCalledWith("official:H2C", true);
  });

  it("keeps slicing and AMS templates separate with live rule and preset controls", async () => {
    const onTemplatesChanged = vi.fn();
    const onRulesChanged = vi.fn();
    const onOutputsChanged = vi.fn();
    const onSavePreset = vi.fn();
    render(NamingPanel, {
      props: {
        presetTemplate: "{clean_name} - {printer_code}",
        amsTemplate: "{vendor} {material} {clean_name}",
        presetRules: [],
        amsRules: [],
        savedPresets: [],
        preview: {
          preset_before: "Panchroma Satin - H2C",
          preset_name: "Panchroma Satin - H2C",
          ams_before: "Polymaker PLA Panchroma Satin",
          ams_name: "Polymaker PLA Panchroma Satin",
        },
        onTemplatesChanged,
        onRulesChanged,
        onOutputsChanged,
        onSavePreset,
      },
    });
    expect(screen.getByText("Panchroma Satin - H2C")).toBeInTheDocument();
    expect(
      screen.getByText("Polymaker PLA Panchroma Satin"),
    ).toBeInTheDocument();
    expect(screen.getByText("Advanced naming rules")).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole("button", {
        name: "Remove {printer_code} from Bambu slicing preset template",
      }),
    );
    expect(onTemplatesChanged).toHaveBeenCalledWith(
      "{clean_name} - ",
      "{vendor} {material} {clean_name}",
    );

    await fireEvent.click(screen.getByText("Advanced naming rules"));
    await fireEvent.input(screen.getByLabelText("Rule pattern"), {
      target: { value: "Polymaker PLA *" },
    });
    await fireEvent.input(screen.getByLabelText("Replacement"), {
      target: { value: "$1" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Add rule" }));
    expect(onRulesChanged).toHaveBeenCalledWith(
      [],
      [
        expect.objectContaining({
          kind: "wildcard",
          pattern: "Polymaker PLA *",
          replacement: "$1",
          case_sensitive: false,
        }),
      ],
    );

    await fireEvent.input(screen.getByLabelText("Naming preset name"), {
      target: { value: "Panchroma clean" },
    });
    await fireEvent.click(
      screen.getByRole("button", { name: "Save naming preset" }),
    );
    expect(onSavePreset).toHaveBeenCalledWith("Panchroma clean");

    await fireEvent.click(screen.getByText("Advanced outputs"));
    await fireEvent.click(
      screen.getByLabelText("Create Bambu slicing presets"),
    );
    expect(onOutputsChanged).toHaveBeenCalledWith({
      slicing_presets: false,
      custom_filaments: true,
    });

    HTMLDialogElement.prototype.showModal = function showModal() {
      this.setAttribute("open", "");
    };
    HTMLDialogElement.prototype.close = function close() {
      this.removeAttribute("open");
    };
    await fireEvent.click(screen.getByRole("button", { name: "Naming help" }));
    const help = screen.getByRole("dialog", {
      name: "Naming templates and rules",
    });
    expect(help).toHaveTextContent(
      "Preview names always come from the migration engine",
    );
    expect(within(help).getAllByRole("button")).toHaveLength(1);
    expect(
      within(help).getByRole("button", { name: "Close" }),
    ).toBeInTheDocument();
  });

  it("inserts every real naming token into the active scoped composer", async () => {
    const onTemplatesChanged = vi.fn();
    render(NamingPanel, {
      props: {
        presetTemplate: "{clean_name}",
        amsTemplate: "{vendor} @{printer_code}",
        presetRules: [],
        amsRules: [],
        savedPresets: [],
        preview: null,
        onTemplatesChanged,
        onRulesChanged: vi.fn(),
        onOutputsChanged: vi.fn(),
        onSavePreset: vi.fn(),
      },
    });

    expect(
      screen.getByRole("button", {
        name: "Insert Source name into active template",
      }),
    ).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole("button", {
        name: "Insert Family into active template",
      }),
    );
    expect(onTemplatesChanged).toHaveBeenLastCalledWith(
      "{clean_name}{family}",
      "{vendor} @{printer_code}",
    );

    await fireEvent.focusIn(
      screen.getByLabelText("AMS custom filament template text 1"),
    );
    expect(
      screen.getByRole("button", {
        name: "Insert Printer code into active template",
      }),
    ).toBeDisabled();
    expect(
      screen.getByText(
        "Remove slicing-only @, {printer_code} from this AMS template.",
      ),
    ).toBeInTheDocument();
  });

  it("shows every target, blocks conflicts, and emits inline resolutions", async () => {
    const onOverride = vi.fn();
    const onDecision = vi.fn();
    render(PlanTable, {
      props: { plan, onOverride, onDecision, conflictDecisions: [] },
    });
    expect(screen.getByText("Bambu Lab H2C 0.4 nozzle")).toBeInTheDocument();
    expect(screen.getByText("Blocked")).toBeInTheDocument();
    expect(screen.getByText("Existing settings differ")).toBeInTheDocument();
    await fireEvent.change(
      screen.getByLabelText("Slicing name for Panchroma PLA Satin"),
      { target: { value: "Satin hand tuned" } },
    );
    expect(onOverride).toHaveBeenCalledWith(
      "op-1",
      "Satin hand tuned",
      "Polymaker PLA Panchroma Satin",
    );
    await fireEvent.change(
      screen.getByLabelText("Conflict decision for Panchroma PLA Satin"),
      { target: { value: "update" } },
    );
    expect(onDecision).toHaveBeenCalledWith("op-1", "update");
  });

  it("reports exact evidence counts without claiming AMS verification", () => {
    const executablePlan: MigrationPlan = {
      ...plan,
      operations: [
        { ...plan.operations[0], action: "create", conflict: null },
        {
          ...plan.operations[0],
          id: "op-skip",
          action: "skip",
          conflict: null,
          nozzle: "0.6",
          printer_preset_name: "Bambu Lab H2C 0.6 nozzle",
        },
      ],
    };
    render(RunProgress, {
      props: {
        plan: executablePlan,
        progress: {
          "op-1": { evidence: "cloud_id_assigned", state: "cloud_id_assigned" },
        },
        running: false,
      },
    });
    expect(screen.getByText("Cloud ID assigned")).toBeInTheDocument();
    expect(screen.getByText("AMS check pending")).toBeInTheDocument();
    expect(screen.getByText("1 / 1 committed")).toBeInTheDocument();
    expect(screen.getByText("1 / 1 acknowledged")).toBeInTheDocument();
    expect(screen.getByText("1 / 1 IDs assigned")).toBeInTheDocument();
    expect(screen.getByText("0 / 1 operator verified")).toBeInTheDocument();
    expect(screen.getByText("Skipped")).toBeInTheDocument();
  });

  it("uses singular grammar for one committed local file", () => {
    render(ResultSummary, {
      props: {
        result: {
          run_id: "run-1",
          plan_id: "plan-1",
          committed_files: 1,
          created_files: 1,
          updated_files: 0,
          deleted_files: 0,
          skipped_operations: 0,
          backup_sha256: "backup-sha256",
          backup_file_count: 1,
          receipt_path: "receipt.json",
        },
      },
    });
    expect(screen.getByText("1 local file committed")).toBeInTheDocument();
    expect(screen.getByText("1 backup file")).toBeInTheDocument();
    expect(screen.getByText("backup-sha256")).toBeInTheDocument();
    expect(screen.queryByText(/checksum-verified/i)).not.toBeInTheDocument();
  });

  it("offers optional support without affecting the completed run", async () => {
    const onOpenSupport = vi
      .fn<() => Promise<void>>()
      .mockRejectedValue(new Error("browser unavailable"));
    render(ResultSummary, {
      props: {
        result: {
          run_id: "run-1",
          plan_id: "plan-1",
          committed_files: 1,
          created_files: 1,
          updated_files: 0,
          deleted_files: 0,
          skipped_operations: 0,
          backup_sha256: "backup-sha256",
          backup_file_count: 1,
          receipt_path: "receipt.json",
        },
        onOpenSupport,
      },
    });

    expect(
      screen.getByText(/Spool Ledger is free and open source/i),
    ).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole("button", { name: "Buy me a coffee" }),
    );
    expect(onOpenSupport).toHaveBeenCalledOnce();
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not open the support page",
    );
    expect(screen.getByText("1 local file committed")).toBeInTheDocument();
  });

  it("links to restore without owning restore mutations", async () => {
    const onOpenRestore = vi.fn();
    render(ResultSummary, {
      props: {
        result: {
          run_id: "run-1",
          plan_id: "plan-1",
          committed_files: 3,
          created_files: 2,
          updated_files: 1,
          deleted_files: 0,
          skipped_operations: 1,
          backup_sha256: "backup-sha256",
          backup_file_count: 3,
          receipt_path: "receipt.json",
        },
        onOpenRestore,
      },
    });

    expect(screen.getByText("3 local files committed")).toBeInTheDocument();
    expect(screen.queryByText("Ready to restore")).not.toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole("button", { name: "Open journal-owned restore" }),
    );
    expect(onOpenRestore).toHaveBeenCalledOnce();
  });
});
