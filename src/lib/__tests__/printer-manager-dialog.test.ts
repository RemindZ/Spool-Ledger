// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { readFileSync } from "node:fs";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import PrinterManagerDialog from "../components/PrinterManagerDialog.svelte";
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

beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function showModal() {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function close() {
    this.removeAttribute("open");
  };
});

afterEach(cleanup);

describe("enabled printer manager", () => {
  it("sizes the dialog itself instead of overflowing a narrower shared surface", () => {
    const root = process.cwd();
    const source = readFileSync(
      `${root}/src/lib/components/PrinterManagerDialog.svelte`,
      "utf8",
    );

    expect(source).toMatch(
      /\.printer-manager-dialog\s*\{[^}]*width:\s*min\(62rem, calc\(100vw - 2rem\)\)/s,
    );
    expect(source).toMatch(
      /\.printer-manager-dialog \.modal-dialog-surface\s*\{[^}]*width:\s*100%/s,
    );
  });

  it("keeps draft changes isolated until Save changes", async () => {
    const onSave = vi.fn();
    const onCancel = vi.fn();
    render(PrinterManagerDialog, {
      open: true,
      catalogId: "targets:1",
      printers,
      enabledPrinterIds: new Set<string>(),
      selectedNozzles: {},
      onSave,
      onCancel,
    });

    const dialog = await screen.findByRole("dialog", {
      name: "Manage enabled printers",
    });
    expect(
      within(dialog).getByRole("button", { name: "Cancel" }),
    ).toHaveFocus();
    await fireEvent.click(
      within(dialog).getByLabelText("Enable Bambu Lab H2C"),
    );
    expect(onSave).not.toHaveBeenCalled();
    await fireEvent.click(
      within(dialog).getByRole("button", { name: "Cancel" }),
    );
    expect(onCancel).toHaveBeenCalledOnce();
    expect(onSave).not.toHaveBeenCalled();
  });

  it("saves enabled printers and default nozzles atomically", async () => {
    const onSave = vi.fn();
    render(PrinterManagerDialog, {
      open: true,
      catalogId: "targets:1",
      printers,
      enabledPrinterIds: new Set<string>(),
      selectedNozzles: {},
      onSave,
      onCancel: vi.fn(),
    });

    const dialog = await screen.findByRole("dialog");
    await fireEvent.click(
      within(dialog).getByLabelText("Enable Bambu Lab H2C"),
    );
    await fireEvent.click(
      within(dialog).getByRole("button", { name: "Save changes" }),
    );

    expect(onSave).toHaveBeenCalledWith(new Set(["official:H2C"]), {
      "official:H2C": new Set(["0.4"]),
    });
  });
});
