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
import { afterEach, describe, expect, it, vi } from "vitest";
import AppChrome from "../components/AppChrome.svelte";
import Harness from "./fixtures/FilamentPickerHarness.svelte";
import type { CatalogSource, DiscoveryResponse } from "../types";

afterEach(cleanup);

function preset(
  id: string,
  vendor: string,
  archetype: string,
  type: string,
  overrides: Partial<CatalogSource> = {},
): CatalogSource {
  return {
    id,
    name: `${vendor} ${archetype} ${type}`,
    vendor,
    material: archetype,
    material_archetype: archetype,
    material_type: type,
    family: `${vendor} ${archetype}`,
    variant: type,
    source_app: "orca_slicer",
    source_kind: "factory_system",
    compatible_printers: ["Bambu Lab H2C 0.4 nozzle"],
    migration_status: "new",
    warnings: [],
    ...overrides,
  };
}

const sources: CatalogSource[] = [
  preset("pla-std", "Overture", "PLA", "Standard"),
  preset("pla-matte", "Overture", "PLA", "Matte"),
  preset("pla-silk", "Fiberlogy", "PLA", "Silk", {
    source_app: "bambu_studio",
  }),
  preset("pla-matte-user", "Fiberlogy", "PLA", "Matte", {
    source_kind: "user_custom",
    warnings: ["Missing parent profile"],
  }),
  preset("abs-std", "Overture", "ABS", "Standard"),
  preset("abs-gf", "Fiberlogy", "ABS", "Glass fiber"),
  preset("pctg", "Fiberlogy", "PCTG", "Standard"),
];

const discovery: DiscoveryResponse = {
  source_root_ids: ["orca-system"],
  manual_source_roots: [],
  target_catalog_ids: ["bambu-system"],
  orca_slicer_detected: true,
  bambu_studio_detected: false,
  platform: "windows",
  accounts: [
    {
      id: "account-ready",
      eligibility: "eligible",
      filament_profile_count: 12,
    },
    { id: "account-backup", eligibility: "backup", filament_profile_count: 50 },
  ],
};

function renderPicker(props: Partial<Record<string, unknown>> = {}) {
  const onSelectionChange = vi.fn();
  const view = render(Harness, {
    props: { sources, discovery, onSelectionChange, ...props },
  });
  return { ...view, onSelectionChange };
}

const card = (name: string) =>
  screen.getByRole("heading", { name }).closest("article") as HTMLElement;

describe("application chrome", () => {
  it("names the empty destination account state instead of rendering a blank select", () => {
    const props = {
      theme: "system" as const,
      onAccountChanged: vi.fn(),
      onThemeChanged: vi.fn(),
    };
    const { unmount } = render(AppChrome, {
      props: {
        ...props,
        discovery: {
          ...discovery,
          accounts: discovery.accounts.filter(
            (account) => account.eligibility !== "eligible",
          ),
        },
        selectedAccountId: null,
      },
    });
    const select = screen.getByLabelText<HTMLSelectElement>(
      "Destination account",
    );
    expect(select.selectedOptions[0]).toHaveTextContent("No eligible account");
    expect(select.selectedOptions[0]).toBeDisabled();
    unmount();

    render(AppChrome, {
      props: { ...props, discovery, selectedAccountId: null },
    });
    expect(
      screen.getByLabelText<HTMLSelectElement>("Destination account")
        .selectedOptions[0],
    ).toHaveTextContent("Select an account");
  });

  it("puts tabs, destination and Local only in one toolbar under the textured title bar", async () => {
    const onAccountChanged = vi.fn();
    const onThemeChanged = vi.fn();
    const { container } = render(AppChrome, {
      props: {
        discovery,
        selectedAccountId: "account-ready",
        theme: "system",
        onAccountChanged,
        onThemeChanged,
      },
    });

    const route = screen.getByLabelText("Migration route");
    expect(route).toHaveTextContent("OrcaSlicer detected");
    expect(route).toHaveTextContent("Bambu Studio not detected");
    expect(container.querySelector(".route-source")).toHaveTextContent(
      "OrcaSlicer",
    );
    expect(screen.queryByText("Sample workspace")).not.toBeInTheDocument();
    expect(screen.queryByText("Source folders")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Appearance" }),
    ).not.toBeInTheDocument();

    const toolbar = container.querySelector(".toolbar") as HTMLElement;
    expect(
      within(toolbar)
        .getAllByRole("tab")
        .map((tab) => tab.textContent?.trim()),
    ).toEqual(["Workspace", "Review plan", "Run & verify", "Restore"]);
    expect(within(toolbar).getByText("Local only")).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("option", { name: /account-backup/ }),
    ).toBeDisabled();
    await fireEvent.change(screen.getByLabelText("Destination account"), {
      target: { value: "account-ready" },
    });
    expect(onAccountChanged).toHaveBeenCalledWith("account-ready");

    await fireEvent.click(
      screen.getByRole("button", { name: /^Theme: Automatic/ }),
    );
    expect(onThemeChanged).toHaveBeenCalledOnce();
  });
});

describe("filament profiles picker", () => {
  it("lays out material cards with the tiering rule and selection meters", async () => {
    renderPicker({ initialSelected: ["pla-matte"] });

    expect(
      [...document.querySelectorAll(".mat-name")].map((h) => h.textContent),
    ).toEqual(["PLA", "PCTG", "ABS"]);
    const pla = card("PLA");
    expect(pla).toHaveClass("has-selection");
    expect(within(pla).getByText("1 selected")).toBeInTheDocument();
    expect(pla).toHaveTextContent("4 presets · 2 manufacturers");
    expect(
      within(pla).getByText("Standard · Matte · Silk"),
    ).toBeInTheDocument();
    expect(
      within(pla).getByRole("button", { name: /Browse 3 types/ }),
    ).toBeInTheDocument();
    expect(
      within(card("ABS")).getByRole("list", { name: "ABS types" }),
    ).toBeInTheDocument();
    expect(
      within(card("PCTG")).getByRole("button", { name: /Browse 1 preset/ }),
    ).toBeInTheDocument();
    expect(
      within(pla).getByRole("checkbox", {
        name: "Select all 4 shown PLA presets",
      }),
    ).toHaveProperty("indeterminate", true);
  });

  it("recomputes tiers under filters and bulk-selects only visible presets", async () => {
    const { onSelectionChange } = renderPicker();
    await fireEvent.click(screen.getByText("All sources"));
    const sourceMenu = screen.getByRole("group", { name: "Preset sources" });
    expect(within(sourceMenu).getByText("OrcaSlicer")).toBeInTheDocument();
    expect(within(sourceMenu).getByText("Bambu Studio")).toBeInTheDocument();
    const bambuSystem = within(sourceMenu).getAllByRole("checkbox")[2];
    await fireEvent.click(bambuSystem);
    expect(screen.getByText("3 of 4")).toBeInTheDocument();
    expect(screen.getByText("3 of 4").closest("details")).toHaveClass(
      "is-filtered",
    );

    const pla = card("PLA");
    expect(within(pla).getByRole("list", { name: "PLA types" })).toBeVisible();
    await fireEvent.click(
      within(pla).getByRole("checkbox", {
        name: "Select all 3 shown PLA presets",
      }),
    );
    expect(onSelectionChange).toHaveBeenLastCalledWith([
      "pla-matte",
      "pla-matte-user",
      "pla-std",
    ]);
    expect(screen.getByText(/presets selected/)).toHaveTextContent(
      "3 presets selected",
    );
  });

  it("scopes manufacturer Select all and Clear all to the search and reports hidden selections", async () => {
    renderPicker({ initialSelected: ["abs-gf", "pla-std"] });
    await fireEvent.click(screen.getByText("All manufacturers"));
    const search = screen.getByLabelText("Filter manufacturers");
    await waitFor(() => expect(search).toHaveFocus());
    await fireEvent.input(search, { target: { value: "fib" } });
    const menu = screen.getByRole("group", { name: "Manufacturers" });
    expect(within(menu).queryByText("Overture")).not.toBeInTheDocument();
    await fireEvent.click(
      within(menu).getByRole("button", { name: "Clear all" }),
    );

    expect(
      screen.getByText("Overture", { selector: ".filter-value" }),
    ).toBeInTheDocument();
    expect(screen.getByText(/presets selected/)).toHaveTextContent(
      "2 presets selected · 1 hidden by filters",
    );
    await fireEvent.input(search, { target: { value: "" } });
    await fireEvent.click(
      within(menu).getByRole("button", { name: "Clear all" }),
    );
    expect(screen.getByText("No manufacturers selected")).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole("button", { name: "Select all manufacturers" }),
    );
    expect(screen.getByText("All manufacturers")).toBeInTheDocument();
  });

  it("drills into materials and types, moves focus, and announces the level", async () => {
    const { container } = renderPicker();
    await fireEvent.click(
      within(card("PLA")).getByRole("button", { name: /Browse 3 types/ }),
    );
    const crumbs = screen.getByRole("navigation", { name: "Material path" });
    await waitFor(() => expect(within(crumbs).getByText("PLA")).toHaveFocus());
    expect(container.querySelector('[aria-live="polite"]')).toHaveTextContent(
      "PLA: 3 types",
    );
    expect(
      (container.querySelector(".picker-stage") as HTMLElement).dataset.motion,
    ).toBe("forward");

    await fireEvent.click(
      within(card("Matte")).getByRole("button", { name: /Browse presets/ }),
    );
    expect(within(crumbs).getByText("Matte")).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(
      screen.getByRole("heading", { name: "Fiberlogy" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Missing parent profile")).toBeInTheDocument();
    expect(container.querySelector(".path-actions")).toHaveTextContent(
      "2 shown · 0 selected",
    );
    await fireEvent.click(screen.getByRole("button", { name: "Select all 2" }));
    expect(
      screen.getByRole("button", { name: "Clear these" }),
    ).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Up one level" }));
    expect(within(crumbs).getByText("PLA")).toHaveAttribute(
      "aria-current",
      "page",
    );
  });

  it("collapses non-tiered materials in the path and skips the missing tier going up", async () => {
    renderPicker();
    await fireEvent.click(
      within(card("ABS")).getByRole("button", { name: /Glass fiber/ }),
    );
    const crumbs = screen.getByRole("navigation", { name: "Material path" });
    expect(within(crumbs).getByText("ABS · Glass fiber")).toHaveAttribute(
      "aria-current",
      "page",
    );
    await fireEvent.click(screen.getByRole("button", { name: "Up one level" }));
    expect(within(crumbs).getByText("All materials")).toHaveAttribute(
      "aria-current",
      "page",
    );
  });

  it("searches within the current level and offers Clear search when nothing matches", async () => {
    renderPicker();
    await fireEvent.click(
      within(card("ABS")).getByRole("button", { name: /Standard/ }),
    );
    const search = screen.getByLabelText("Search presets in this view");
    await fireEvent.input(search, { target: { value: "matte" } });
    expect(screen.getByText("No presets match “matte”")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Clear search" }));
    expect(search).toHaveValue("");
    expect(
      screen.getByLabelText("Select Overture ABS Standard"),
    ).toBeInTheDocument();

    await fireEvent.input(search, { target: { value: "matte" } });
    await fireEvent.click(screen.getByRole("button", { name: "Up one level" }));
    expect(
      screen.getByRole("heading", { name: "PLA · Matte" }),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Select Overture PLA Matte"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Select Fiberlogy PLA Matte"),
    ).toBeInTheDocument();
  });

  it("shows the selected-only empty state and the footer popovers", async () => {
    const onRefresh = vi.fn();
    const onSavePreset = vi.fn();
    renderPicker({ onRefresh, onSavePreset });

    await fireEvent.click(screen.getByLabelText("Show selected only"));
    expect(screen.getByText("Nothing selected here yet")).toBeInTheDocument();

    await fireEvent.click(screen.getByText("Catalog"));
    expect(screen.getByText("Ready · 7 presets")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Source folders" }),
    ).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole("button", { name: "Refresh profile discovery" }),
    );
    expect(onRefresh).toHaveBeenCalledOnce();

    await fireEvent.click(screen.getByText("Saved filters"));
    await fireEvent.input(screen.getByLabelText("Filter preset name"), {
      target: { value: "Everyday" },
    });
    await fireEvent.click(
      screen.getByRole("button", { name: "Save filter preset" }),
    );
    expect(onSavePreset).toHaveBeenCalledWith("Everyday");
    expect(screen.queryByLabelText("Show custom printers")).toBeNull();
  });
});

describe("drilldown history", () => {
  async function drillIntoMatte() {
    await fireEvent.click(
      within(card("PLA")).getByRole("button", { name: /Browse 3 types/ }),
    );
    await fireEvent.click(
      within(card("Matte")).getByRole("button", { name: /Browse presets/ }),
    );
  }

  const current = () =>
    within(screen.getByRole("navigation", { name: "Material path" }))
      .getAllByText(/./)
      .find((node) => node.getAttribute("aria-current") === "page")
      ?.textContent;

  it("retraces levels with mouse Back and Forward and cancels the default", async () => {
    renderPicker();
    await drillIntoMatte();
    expect(current()).toBe("Matte");

    const down = new MouseEvent("mousedown", {
      button: 3,
      bubbles: true,
      cancelable: true,
    });
    document.dispatchEvent(down);
    expect(down.defaultPrevented).toBe(true);
    const up = new MouseEvent("mouseup", {
      button: 3,
      bubbles: true,
      cancelable: true,
    });
    document.dispatchEvent(up);
    expect(up.defaultPrevented).toBe(true);
    await waitFor(() => expect(current()).toBe("PLA"));

    document.dispatchEvent(new MouseEvent("mouseup", { button: 3 }));
    await waitFor(() => expect(current()).toBe("All materials"));
    document.dispatchEvent(new MouseEvent("mouseup", { button: 3 }));
    expect(current()).toBe("All materials");

    document.dispatchEvent(new MouseEvent("mouseup", { button: 4 }));
    await waitFor(() => expect(current()).toBe("PLA"));
  });

  it("moves with Alt+arrows and clears Forward after a new navigation", async () => {
    renderPicker();
    await drillIntoMatte();
    await fireEvent.keyDown(document, { key: "ArrowLeft", altKey: true });
    await waitFor(() => expect(current()).toBe("PLA"));

    await fireEvent.click(
      within(card("Silk")).getByRole("button", { name: /Browse presets/ }),
    );
    expect(current()).toBe("Silk");
    await fireEvent.keyDown(document, { key: "ArrowRight", altKey: true });
    expect(current()).toBe("Silk");
    await fireEvent.keyDown(document, { key: "ArrowLeft", altKey: true });
    await waitFor(() => expect(current()).toBe("PLA"));
  });

  it("does nothing while the step is not in front", async () => {
    renderPicker({ historyActive: false });
    await drillIntoMatte();
    const up = new MouseEvent("mouseup", {
      button: 3,
      bubbles: true,
      cancelable: true,
    });
    document.dispatchEvent(up);
    expect(up.defaultPrevented).toBe(false);
    await fireEvent.keyDown(document, { key: "ArrowLeft", altKey: true });
    expect(current()).toBe("Matte");
  });
});
