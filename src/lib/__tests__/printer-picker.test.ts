// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import PrinterPicker from "../components/PrinterPicker.svelte";
import type { PrinterTarget } from "../types";

const printers: PrinterTarget[] = [
  {
    id: "official:H2C",
    name: "Bambu Lab H2C",
    code: "H2C",
    kind: "official",
    verified: true,
    artwork_available: false,
    extruder_variants: ["Direct Drive Standard"],
    nozzles: ["0.2", "0.4", "0.6"].map((diameter) => ({
      id: `H2C:${diameter}`,
      diameter,
      printer_preset_name: `Bambu Lab H2C ${diameter} nozzle`,
      selected: false,
      supported: true,
    })),
  },
  {
    id: "official:Legacy",
    name: "Bambu Lab Legacy",
    code: "Legacy",
    kind: "official",
    verified: true,
    artwork_available: false,
    extruder_variants: ["Direct Drive Standard"],
    nozzles: ["0.6", "0.8"].map((diameter) => ({
      id: `Legacy:${diameter}`,
      diameter,
      printer_preset_name: `Bambu Lab Legacy ${diameter} nozzle`,
      selected: false,
      supported: true,
    })),
  },
  {
    id: "custom:Workshop",
    name: "Workshop CoreXY",
    code: "Workshop",
    kind: "custom",
    verified: false,
    artwork_available: false,
    extruder_variants: [],
    nozzles: [],
  },
];

afterEach(cleanup);

describe("official printer picker", () => {
  it("enables 0.4 mm by default and never exposes custom printers", async () => {
    const onChange = vi.fn();
    render(PrinterPicker, {
      catalogId: "targets:1",
      printers,
      enabledPrinterIds: new Set<string>(),
      selectedNozzles: {},
      onChange,
    });

    expect(screen.queryByText("Workshop CoreXY")).not.toBeInTheDocument();
    await fireEvent.click(screen.getByLabelText("Enable Bambu Lab H2C"));

    expect(onChange).toHaveBeenCalledWith(new Set(["official:H2C"]), {
      "official:H2C": new Set(["0.4"]),
    });
  });

  it("falls back to the first supported nozzle when 0.4 mm is unavailable", async () => {
    const onChange = vi.fn();
    render(PrinterPicker, {
      catalogId: "targets:1",
      printers,
      enabledPrinterIds: new Set<string>(),
      selectedNozzles: {},
      onChange,
    });

    await fireEvent.click(screen.getByLabelText("Enable Bambu Lab Legacy"));

    expect(onChange).toHaveBeenCalledWith(new Set(["official:Legacy"]), {
      "official:Legacy": new Set(["0.6"]),
    });
  });
});
