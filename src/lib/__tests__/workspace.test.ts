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
import AppChrome from "../components/AppChrome.svelte";
import FilterBar from "../components/FilterBar.svelte";
import FilterTools from "../components/FilterTools.svelte";
import SourceTable from "../components/SourceTable.svelte";
import { createInitialState, sourceFacets, type SourceFilters } from "../state";
import type { CatalogSource, DiscoveryResponse } from "../types";

afterEach(cleanup);

const sources: CatalogSource[] = [
  {
    id: "orca:satin",
    name: "Panchroma PLA Satin @BBL X1C",
    vendor: "Polymaker",
    material: "PLA",
    family: "Panchroma",
    variant: "Satin",
    source_app: "orca_slicer",
    source_kind: "factory_system",
    compatible_printers: ["Bambu Lab X1 Carbon"],
    migration_status: "new",
    warnings: [],
  },
  {
    id: "bambu:petg",
    name: "Workshop PETG",
    vendor: "Workshop",
    material: "PETG",
    family: "Utility",
    variant: "Matte",
    source_app: "bambu_studio",
    source_kind: "user_custom",
    compatible_printers: ["Bambu Lab H2C"],
    migration_status: "conflicting",
    warnings: ["Missing characterized field policy"],
  },
];

const discovery: DiscoveryResponse = {
  source_root_ids: ["orca-system", "bambu-user"],
  manual_source_roots: [],
  target_catalog_ids: ["bambu-system"],
  orca_slicer_detected: true,
  bambu_studio_detected: true,
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

describe("workspace controls", () => {
  it("names the empty destination account state instead of rendering a blank select", () => {
    const props = {
      theme: "system" as const,
      busy: false,
      onRefresh: vi.fn(),
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

  it("shows truthful discovery, eligible accounts, source access, and the theme menu", async () => {
    const onAccountChanged = vi.fn();
    const onThemeChanged = vi.fn();
    render(AppChrome, {
      props: {
        discovery,
        selectedAccountId: "account-ready",
        theme: "system",
        busy: false,
        onRefresh: vi.fn(),
        onAccountChanged,
        onThemeChanged,
      },
    });

    const detected = screen.getByLabelText("Detected applications");
    expect(detected).toHaveTextContent("OrcaSlicer detected");
    expect(detected).toHaveTextContent("Bambu Studio detected");
    expect(screen.queryByText(/Windows build/i)).not.toBeInTheDocument();
    expect(screen.getByText("Local only")).toBeInTheDocument();
    expect(
      screen.queryByText(/Nothing is written until you review/),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Source folders")).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /Setup/ })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: /account-ready/ })).toBeEnabled();
    expect(
      screen.getByRole("option", { name: /account-backup/ }),
    ).toBeDisabled();
    await fireEvent.change(screen.getByLabelText("Destination account"), {
      target: { value: "account-ready" },
    });
    expect(onAccountChanged).toHaveBeenCalledWith("account-ready");

    const appearance = screen.getByRole("button", { name: "Appearance" });
    expect(appearance).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.queryByRole("group", { name: "Theme" }),
    ).not.toBeInTheDocument();
    await fireEvent.click(appearance);
    expect(appearance).toHaveAttribute("aria-expanded", "true");
    const theme = screen.getByRole("group", { name: "Theme" });
    expect(
      within(theme).getByRole("button", { name: "System" }),
    ).toHaveAttribute("aria-pressed", "true");
    await fireEvent.click(within(theme).getByRole("button", { name: "Dark" }));
    expect(onThemeChanged).toHaveBeenCalledWith("dark");
    await fireEvent.keyDown(
      within(theme).getByRole("button", { name: "Light" }),
      {
        key: "Escape",
      },
    );
    expect(appearance).toHaveAttribute("aria-expanded", "false");
  });

  it("exposes every filter dimension through chips and More filters", async () => {
    const filters = createInitialState().filters;
    const onFiltersChanged = vi.fn();
    const onReset = vi.fn();
    render(FilterBar, {
      props: {
        filters,
        facets: sourceFacets(sources),
        onFiltersChanged,
        onReset,
      },
    });

    await fireEvent.input(screen.getByLabelText("Search source profiles"), {
      target: { value: "satin" },
    });
    expect(onFiltersChanged).toHaveBeenCalledWith({ search: "satin" });

    expect(screen.queryByLabelText("OrcaSlicer")).not.toBeInTheDocument();
    const appChip = screen.getByRole("button", { name: "Source app" });
    await fireEvent.click(appChip);
    expect(appChip).toHaveAttribute("aria-expanded", "true");
    await fireEvent.click(screen.getByLabelText("OrcaSlicer"));
    const update = onFiltersChanged.mock.calls[
      onFiltersChanged.mock.calls.length - 1
    ]?.[0] as Partial<SourceFilters>;
    expect(update.sourceApps).toEqual(new Set(["orca_slicer"]));
    expect(filters.sourceApps.size).toBe(0);
    await fireEvent.keyDown(screen.getByLabelText("OrcaSlicer"), {
      key: "Escape",
    });
    expect(appChip).toHaveAttribute("aria-expanded", "false");

    for (const name of ["Manufacturer", "Material", "Status"]) {
      expect(screen.getByRole("button", { name })).toBeInTheDocument();
    }
    await fireEvent.click(screen.getByRole("button", { name: "More filters" }));
    const more = screen.getByRole("group", { name: "More filters" });
    for (const title of [
      "Source kind",
      "Family",
      "Variant",
      "Compatible printer",
    ]) {
      expect(within(more).getByText(title)).toBeInTheDocument();
    }
    await fireEvent.click(within(more).getByLabelText("User / custom"));
    expect(
      (
        onFiltersChanged.mock.calls[
          onFiltersChanged.mock.calls.length - 1
        ]?.[0] as Partial<SourceFilters>
      ).sourceKinds,
    ).toEqual(new Set(["user_custom"]));

    await fireEvent.click(
      screen.getByRole("button", { name: "Reset filters" }),
    );
    expect(onReset).toHaveBeenCalledOnce();
  });

  it("counts active selections on each filter chip", () => {
    const filters = {
      ...createInitialState().filters,
      materials: new Set(["PLA"]),
      families: new Set(["Panchroma"]),
      variants: new Set(["Satin"]),
    };
    render(FilterBar, {
      props: {
        filters,
        facets: sourceFacets(sources),
        onFiltersChanged: vi.fn(),
        onReset: vi.fn(),
      },
    });

    expect(
      screen.getByRole("button", { name: "Material, 1 selected" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "More filters, 2 selected" }),
    ).toBeInTheDocument();
  });

  it("keeps selected-only and saved filter presets beside the selection controls", async () => {
    const filters = createInitialState().filters;
    const onFiltersChanged = vi.fn();
    const onSavePreset = vi.fn();
    const onLoadPreset = vi.fn();
    const onDeletePreset = vi.fn();
    render(FilterTools, {
      props: {
        filters,
        savedPresets: [{ id: "pla", name: "PLA only", filters }],
        onFiltersChanged,
        onSavePreset,
        onLoadPreset,
        onDeletePreset,
      },
    });

    await fireEvent.click(screen.getByLabelText("Selected only"));
    expect(onFiltersChanged).toHaveBeenCalledWith({ selectedOnly: true });

    await fireEvent.click(screen.getByText("Saved filters"));
    await fireEvent.input(screen.getByLabelText("Filter preset name"), {
      target: { value: "Panchroma" },
    });
    await fireEvent.click(
      screen.getByRole("button", { name: "Save filter preset" }),
    );
    expect(onSavePreset).toHaveBeenCalledWith("Panchroma");
    await fireEvent.change(screen.getByLabelText("Saved filter presets"), {
      target: { value: "pla" },
    });
    expect(onLoadPreset).toHaveBeenCalledWith("pla");
    await fireEvent.click(
      screen.getByRole("button", { name: "Delete filter preset PLA only" }),
    );
    expect(onDeletePreset).toHaveBeenCalledWith("pla");
  });

  it("renders dense source provenance, warnings, and selection controls", async () => {
    const onToggle = vi.fn();
    const onSelectVisible = vi.fn();
    render(SourceTable, {
      props: {
        sources,
        selectedIds: new Set(["orca:satin"]),
        totalCount: 7,
        onToggle,
        onSelectVisible,
        onClearSelection: vi.fn(),
      },
    });

    expect(screen.getByText("2 of 7 profiles")).toBeInTheDocument();
    const satinRow = screen.getByRole("row", { name: /Panchroma PLA Satin/ });
    expect(within(satinRow).getByText("OrcaSlicer")).toBeInTheDocument();
    expect(within(satinRow).getByRole("checkbox")).toBeChecked();
    expect(
      screen.getByText("Missing characterized field policy"),
    ).toBeInTheDocument();
    await fireEvent.click(within(satinRow).getByRole("checkbox"));
    expect(onToggle).toHaveBeenCalledWith("orca:satin");
    await fireEvent.click(
      screen.getByRole("button", { name: "Select visible" }),
    );
    expect(onSelectVisible).toHaveBeenCalledWith(true);
  });
});
