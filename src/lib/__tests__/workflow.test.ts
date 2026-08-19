// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
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
    extruder_variants: ["Direct Drive Standard", "Direct Drive High Flow"],
    nozzles: ["0.2", "0.4"].map((diameter) => ({
      id: `H2C:${diameter}`,
      diameter,
      printer_preset_name: `Bambu Lab H2C ${diameter} nozzle`,
      selected: false,
      supported: true,
    })),
  },
  {
    id: "custom:Workshop",
    name: "Workshop CoreXY",
    code: "WC",
    kind: "custom",
    verified: false,
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
      source_settings_fingerprint: "a",
      precondition_fingerprint: null,
      action: "block",
      conflict: {
        kind: "settings_mismatch",
        message: "Existing settings differ",
      },
    },
  ],
};

describe("workflow panels", () => {
  it("hides custom printers until explicitly enabled and marks them unverified", async () => {
    const onNozzleToggle = vi.fn();
    const onSelectAll = vi.fn();
    const { rerender } = render(TargetPanel, {
      props: {
        printers,
        selectedNozzles: {},
        showCustom: false,
        onNozzleToggle,
        onSelectAll,
      },
    });
    expect(screen.queryByText("Workshop CoreXY")).not.toBeInTheDocument();
    await rerender({
      printers,
      selectedNozzles: {},
      showCustom: true,
      onNozzleToggle,
      onSelectAll,
    });
    expect(screen.getByText("Workshop CoreXY")).toBeInTheDocument();
    expect(screen.getByText("Unverified custom printer")).toBeInTheDocument();
  });

  it("selects nozzles independently and exposes explicit select-all", async () => {
    const onNozzleToggle = vi.fn();
    const onSelectAll = vi.fn();
    render(TargetPanel, {
      props: {
        printers,
        selectedNozzles: {},
        showCustom: false,
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

  it("keeps slicing and AMS templates separate with live examples", async () => {
    const onTemplatesChanged = vi.fn();
    render(NamingPanel, {
      props: {
        presetTemplate: "{clean_name} - {printer_code}",
        amsTemplate: "{vendor} {material} {clean_name}",
        preview: {
          preset_name: "Panchroma Satin - H2C",
          ams_name: "Polymaker PLA Panchroma Satin",
        },
        onTemplatesChanged,
      },
    });
    expect(screen.getByText("Panchroma Satin - H2C")).toBeInTheDocument();
    expect(
      screen.getByText("Polymaker PLA Panchroma Satin"),
    ).toBeInTheDocument();
    expect(screen.getByText("Advanced naming rules")).toBeInTheDocument();
    await fireEvent.input(
      screen.getByLabelText("Bambu slicing preset template"),
      {
        target: { value: "{clean_name}" },
      },
    );
    expect(onTemplatesChanged).toHaveBeenCalled();
  });

  it("shows every target and defaults conflicts to blocked", () => {
    render(PlanTable, { props: { plan } });
    expect(screen.getByText("Bambu Lab H2C 0.4 nozzle")).toBeInTheDocument();
    expect(screen.getByText("Blocked")).toBeInTheDocument();
    expect(screen.getByText("Existing settings differ")).toBeInTheDocument();
  });

  it("does not claim AMS verification from a cloud ID", () => {
    render(RunProgress, {
      props: {
        plan,
        progress: {
          "op-1": { evidence: "cloud_id_assigned", state: "cloud_id_assigned" },
        },
        running: false,
      },
    });
    expect(screen.getByText("Cloud ID assigned")).toBeInTheDocument();
    expect(screen.getByText("AMS check pending")).toBeInTheDocument();
  });

  it("previews restore impact and preserves externally changed paths", () => {
    render(ResultSummary, {
      props: {
        result: {
          run_id: "run-1",
          plan_id: "plan-1",
          committed_files: 3,
          receipt_path: "receipt.json",
        },
        restorePreview: {
          paths: [
            {
              path: "filament/base/changed.info",
              action: "update",
              safe_to_restore: false,
              current_sha256: "external",
              expected_committed_sha256: "owned",
            },
          ],
        },
      },
    });
    expect(screen.getByText("3 local files committed")).toBeInTheDocument();
    expect(
      screen.getByText("Externally changed - preserve"),
    ).toBeInTheDocument();
  });
});
