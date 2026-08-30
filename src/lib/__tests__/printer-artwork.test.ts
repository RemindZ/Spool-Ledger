// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ printerArtwork: vi.fn() }));
vi.mock("../api", () => ({ api: mocks }));

import PrinterArtwork from "../components/PrinterArtwork.svelte";
import type { PrinterTarget } from "../types";

const printer: PrinterTarget = {
  id: "official:H2C",
  name: "Bambu Lab H2C",
  code: "H2C",
  kind: "official",
  verified: true,
  artwork_available: true,
  extruder_variants: ["Direct Drive Standard"],
  nozzles: [],
};

beforeEach(() => {
  vi.clearAllMocks();
  URL.createObjectURL = vi.fn(() => "blob:printer-artwork");
  URL.revokeObjectURL = vi.fn();
});

afterEach(cleanup);

describe("printer artwork", () => {
  it("loads available artwork by opaque ids and revokes its blob URL", async () => {
    mocks.printerArtwork.mockResolvedValue(
      new Uint8Array([137, 80, 78, 71]).buffer,
    );
    const view = render(PrinterArtwork, { catalogId: "targets:1", printer });

    const image = await screen.findByRole("img", { name: "Bambu Lab H2C" });
    expect(image).toHaveAttribute("src", "blob:printer-artwork");
    expect(mocks.printerArtwork).toHaveBeenCalledWith(
      "targets:1",
      "official:H2C",
    );

    view.unmount();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:printer-artwork");
  });

  it("uses the local printer glyph when artwork is unavailable", async () => {
    render(PrinterArtwork, {
      catalogId: "targets:1",
      printer: { ...printer, artwork_available: false },
    });

    await waitFor(() => expect(mocks.printerArtwork).not.toHaveBeenCalled());
    expect(screen.getByTestId("printer-artwork-fallback")).toBeInTheDocument();
  });
});
