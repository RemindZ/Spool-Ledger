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
import PlanDependencyDialog from "../components/PlanDependencyDialog.svelte";
import type { TargetTemplateIssue } from "../types";

const issue: TargetTemplateIssue = {
  id: "target-template:pet-cf:h2c:0.4",
  expected_name: "Generic PET-CF @BBL H2C",
  material: "PET-CF",
  printer_id: "official:H2C",
  printer_name: "Bambu Lab H2C",
  nozzle: "0.4",
  affected_sources: [
    { id: "elegoo-pet-cf", name: "Elegoo PET-CF" },
    { id: "other-pet-cf", name: "Other PET-CF" },
  ],
  installed_candidates: [
    { profile_name: "Bambu PET-CF @BBL H2C", recommended: true },
  ],
  source_candidates: [
    { source_id: "bambu-user-pet-cf", name: "My Bambu PET-CF" },
  ],
  diagnostic: "The expected generic target is not installed.",
};

beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function showModal() {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function close() {
    this.removeAttribute("open");
  };
});

afterEach(cleanup);

describe("target profile dependency dialog", () => {
  it("sizes the dialog itself so long dependency actions cannot scroll it sideways", () => {
    const source = readFileSync(
      `${process.cwd()}/src/lib/components/PlanDependencyDialog.svelte`,
      "utf8",
    );

    expect(source).toMatch(
      /\.dependency-dialog\s*\{[^}]*width:\s*min\(46rem, calc\(100vw - 2rem\)\)/s,
    );
    expect(source).toMatch(
      /\.dependency-dialog \.modal-dialog-surface\s*\{[^}]*width:\s*100%/s,
    );
  });

  it("names every affected filament and returns an explicit installed decision", async () => {
    const onResolve = vi.fn();
    render(PlanDependencyDialog, {
      open: true,
      issues: [issue],
      busy: false,
      onCancel: vi.fn(),
      onResolve,
      onRemove: vi.fn(),
    });

    const dialog = await screen.findByRole("dialog", {
      name: "Target profile required",
    });
    expect(
      within(dialog).getByRole("button", { name: "Cancel" }),
    ).toHaveFocus();
    expect(dialog).toHaveTextContent("Generic PET-CF @BBL H2C");
    expect(dialog).toHaveTextContent("Elegoo PET-CF");
    expect(dialog).toHaveTextContent("Other PET-CF");
    expect(dialog).toHaveTextContent(
      "The app will not fabricate a target profile from an unverified source.",
    );

    await fireEvent.click(
      within(dialog).getByRole("radio", {
        name: /Use Bambu PET-CF @BBL H2C for Bambu Lab H2C/,
      }),
    );
    await fireEvent.click(
      within(dialog).getByRole("button", { name: "Resolve dependencies" }),
    );

    expect(onResolve).toHaveBeenCalledWith([
      {
        issue_id: issue.id,
        action: "use_installed",
        profile_name: "Bambu PET-CF @BBL H2C",
        source_id: null,
      },
    ]);
  });

  it("offers safe source inclusion and removes all affected filaments as one action", async () => {
    const onResolve = vi.fn();
    const onRemove = vi.fn();
    render(PlanDependencyDialog, {
      open: true,
      issues: [issue],
      busy: false,
      onCancel: vi.fn(),
      onResolve,
      onRemove,
    });

    const dialog = await screen.findByRole("dialog");
    await fireEvent.click(
      within(dialog).getByRole("radio", {
        name: /Include and migrate My Bambu PET-CF/,
      }),
    );
    await fireEvent.click(
      within(dialog).getByRole("button", { name: "Resolve dependencies" }),
    );
    expect(onResolve).toHaveBeenCalledWith([
      {
        issue_id: issue.id,
        action: "use_source",
        profile_name: null,
        source_id: "bambu-user-pet-cf",
      },
    ]);

    await fireEvent.click(
      within(dialog).getByRole("button", {
        name: "Remove 2 affected filaments",
      }),
    );
    expect(onRemove).toHaveBeenCalledWith(["elegoo-pet-cf", "other-pet-cf"]);
  });
});
