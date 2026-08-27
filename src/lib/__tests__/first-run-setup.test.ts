// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import FirstRunSetup from "../components/FirstRunSetup.svelte";
import type { CatalogSource, PrinterTarget } from "../types";

const sources: CatalogSource[] = [
  {
    id: "orca-system-pla",
    name: "Orca PLA",
    vendor: "Generic",
    material: "PLA",
    family: "PLA",
    variant: "",
    source_app: "orca_slicer",
    source_kind: "factory_system",
    compatible_printers: [],
    migration_status: "new",
    warnings: [],
  },
  {
    id: "bambu-user-pla",
    name: "Bambu user PLA",
    vendor: "User",
    material: "PLA",
    family: "PLA",
    variant: "",
    source_app: "bambu_studio",
    source_kind: "user_custom",
    compatible_printers: [],
    migration_status: "new",
    warnings: [],
  },
];

const printers: PrinterTarget[] = [
  {
    id: "official:H2C",
    name: "Bambu Lab H2C",
    code: "H2C",
    kind: "official",
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
  },
];

afterEach(cleanup);

describe("first-run setup", () => {
  it("uses real source counts and completes with explicit filters and printer defaults", async () => {
    const onComplete = vi.fn();
    render(FirstRunSetup, {
      catalogId: "targets:1",
      sources,
      printers,
      onComplete,
    });

    expect(screen.getByLabelText("OrcaSlicer, 1 profile")).toBeChecked();
    expect(screen.getByLabelText("Bambu Studio, 1 profile")).toBeChecked();
    const continueButton = screen.getByRole("button", {
      name: "Continue to migration",
    });
    expect(continueButton).toBeDisabled();

    await fireEvent.click(screen.getByLabelText("Enable Bambu Lab H2C"));
    expect(continueButton).toBeEnabled();
    await fireEvent.click(continueButton);

    expect(onComplete).toHaveBeenCalledWith({
      sourceApps: new Set(["orca_slicer", "bambu_studio"]),
      sourceKinds: new Set(["factory_system", "user_custom"]),
      enabledPrinterIds: new Set(["official:H2C"]),
      selectedNozzles: { "official:H2C": new Set(["0.4"]) },
    });
  });

  it("blocks a source application and profile-type combination with no real profiles", async () => {
    render(FirstRunSetup, {
      catalogId: "targets:1",
      sources,
      printers,
      onComplete: vi.fn(),
    });

    await fireEvent.click(screen.getByLabelText("OrcaSlicer, 1 profile"));
    await fireEvent.click(screen.getByLabelText("User / custom, 1 profile"));
    await fireEvent.click(screen.getByLabelText("Enable Bambu Lab H2C"));

    expect(
      screen.getByText(
        "No discovered profiles match this source and profile-type combination.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Continue to migration" }),
    ).toBeDisabled();
  });
});
