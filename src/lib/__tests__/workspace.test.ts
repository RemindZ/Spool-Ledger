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
import DiscoveryBar from "../components/DiscoveryBar.svelte";
import FilterPanel from "../components/FilterPanel.svelte";
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
  target_catalog_ids: ["bambu-system"],
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
  it("shows discovery counts, limits account choices to eligible roots, and changes theme", async () => {
    const onAccountChanged = vi.fn();
    const onThemeChanged = vi.fn();
    render(DiscoveryBar, {
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

    expect(screen.getByText("2 source roots")).toBeInTheDocument();
    expect(screen.getByRole("option", { name: /account-ready/ })).toBeEnabled();
    expect(
      screen.getByRole("option", { name: /account-backup/ }),
    ).toBeDisabled();
    await fireEvent.change(screen.getByLabelText("Bambu account"), {
      target: { value: "account-ready" },
    });
    expect(onAccountChanged).toHaveBeenCalledWith("account-ready");
    await fireEvent.change(screen.getByLabelText("Theme"), {
      target: { value: "dark" },
    });
    expect(onThemeChanged).toHaveBeenCalledWith("dark");
  });

  it("exposes every filter dimension and emits immutable filter updates", async () => {
    const filters = createInitialState().filters;
    const onFiltersChanged = vi.fn();
    const onSavePreset = vi.fn();
    const onLoadPreset = vi.fn();
    render(FilterPanel, {
      props: {
        filters,
        facets: sourceFacets(sources),
        savedPresets: [{ id: "pla", name: "PLA only", filters }],
        onFiltersChanged,
        onReset: vi.fn(),
        onSavePreset,
        onLoadPreset,
      },
    });

    await fireEvent.input(screen.getByLabelText("Search source profiles"), {
      target: { value: "satin" },
    });
    expect(onFiltersChanged).toHaveBeenCalledWith({ search: "satin" });

    await fireEvent.click(screen.getByLabelText("OrcaSlicer"));
    const update = onFiltersChanged.mock.calls.at(
      -1,
    )?.[0] as Partial<SourceFilters>;
    expect(update.sourceApps).toEqual(new Set(["orca_slicer"]));
    expect(filters.sourceApps.size).toBe(0);

    expect(screen.getByText("Manufacturer")).toBeInTheDocument();
    expect(screen.getByText("Material")).toBeInTheDocument();
    expect(screen.getByText("Family")).toBeInTheDocument();
    expect(screen.getByText("Variant")).toBeInTheDocument();
    expect(screen.getByText("Compatible printer")).toBeInTheDocument();
    expect(screen.getByText("Migration status")).toBeInTheDocument();
    expect(screen.getByLabelText("Selected only")).toBeInTheDocument();

    await fireEvent.click(screen.getByText("Saved filter presets"));
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
