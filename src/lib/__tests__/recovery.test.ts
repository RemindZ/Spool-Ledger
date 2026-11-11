// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import RestorePanel from "../components/RestorePanel.svelte";
import RunRail from "../components/RunRail.svelte";
import RunStatus from "../components/RunStatus.svelte";
import { runRail } from "../run-rail";
import type { LocalRunResult } from "../types";

const runResult = (committed: number): LocalRunResult => ({
  run_id: "run-1",
  plan_id: "plan-1",
  committed_files: committed,
  created_files: committed,
  updated_files: 0,
  deleted_files: 0,
  skipped_operations: 0,
  backup_sha256: "backup-sha256",
  backup_file_count: committed,
  receipt_path: "receipt.json",
});

beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function showModal() {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function close() {
    this.removeAttribute("open");
  };
});

afterEach(cleanup);

describe("RunStatus", () => {
  it("leads with a timeout verdict and the actions that resolve it", async () => {
    const onRetrySync = vi.fn();
    const onOpenRestore = vi.fn();
    render(RunStatus, {
      props: {
        result: runResult(6),
        synchronization: {
          timed_out: true,
          highest_evidence: "created_local",
          observations: [],
        },
        onRetrySync,
        onOpenRestore,
      },
    });

    expect(
      screen.getByRole("heading", { name: "Synchronization timed out" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "6 local files remain committed. The highest observed evidence is created locally.",
      ),
    ).toBeInTheDocument();
    await fireEvent.click(
      screen.getByRole("button", { name: "Retry synchronization" }),
    );
    await fireEvent.click(
      screen.getByRole("button", { name: "Open journal-owned restore" }),
    );
    expect(onRetrySync).toHaveBeenCalledOnce();
    expect(onOpenRestore).toHaveBeenCalledOnce();
  });

  it("reports a stopped synchronization with its diagnostic", () => {
    render(RunStatus, {
      props: {
        result: runResult(1),
        syncError: "Bambu Studio did not start",
      },
    });

    expect(
      screen.getByRole("heading", { name: "Synchronization stopped" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "1 local file remains committed. Bambu Studio did not start",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Retry synchronization" }),
    ).toBeInTheDocument();
  });

  it("shows synchronization in progress without offering a retry", () => {
    render(RunStatus, {
      props: {
        result: runResult(2),
        syncing: true,
      },
    });

    expect(
      screen.getByRole("heading", { name: "Synchronizing with Bambu Studio" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Retry synchronization" }),
    ).not.toBeInTheDocument();
  });

  it("keeps AMS verification operator-only after observed synchronization", () => {
    render(RunStatus, {
      props: {
        result: runResult(2),
        synchronization: {
          timed_out: false,
          highest_evidence: "cloud_id_assigned",
          observations: [],
        },
      },
    });

    expect(
      screen.getByRole("heading", { name: "Synchronization observed" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "2 local files committed. The highest observed evidence is cloud ID assigned. AMS verification remains operator-only.",
      ),
    ).toBeInTheDocument();
  });

  it("uses singular grammar for one committed local file", () => {
    render(RunStatus, {
      props: { result: runResult(1) },
    });

    expect(
      screen.getByRole("heading", { name: "1 local file committed" }),
    ).toBeInTheDocument();
    expect(screen.getByText("run-1")).toBeInTheDocument();
  });

  it("waits for a committed run before anything starts", () => {
    render(RunStatus, { props: { result: null } });

    expect(
      screen.getByRole("heading", { name: "Waiting for a committed run" }),
    ).toBeInTheDocument();
    expect(screen.getByText("No committed run")).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("reports a failed commit without claiming any committed files", () => {
    render(RunStatus, {
      props: {
        result: null,
        localError: "Staged output failed validation.",
      },
    });

    expect(
      screen.getByRole("heading", { name: "Local transaction stopped" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Staged output failed validation. Review the plan, then commit again.",
      ),
    ).toBeInTheDocument();
  });

  it("describes a running local transaction without actions", () => {
    render(RunStatus, {
      props: { result: null, executing: true },
    });

    expect(
      screen.getByRole("heading", { name: "Committing local profiles" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});

describe("RunRail", () => {
  const local = [
    { phase: "validating", elapsed_ms: 0 },
    { phase: "closing_bambu", elapsed_ms: 120 },
    { phase: "staging", elapsed_ms: 300 },
  ] as const;

  it("shows the idle rail before a transaction", () => {
    const { container } = render(RunRail, {
      props: {
        rail: runRail({
          local: [],
          sync: [],
          executing: false,
          localFailed: false,
          syncing: false,
          syncExpected: true,
          syncStopped: false,
        }),
      },
    });

    expect(screen.getByText("No transaction started")).toBeInTheDocument();
    expect(container.querySelectorAll(".phase")).toHaveLength(10);
    expect(
      container.querySelectorAll(".phase.done, .phase.active"),
    ).toHaveLength(0);
  });

  it("times only completed steps and highlights the active one", () => {
    const { container } = render(RunRail, {
      props: {
        rail: runRail({
          local: [...local],
          sync: [],
          executing: true,
          localFailed: false,
          syncing: false,
          syncExpected: true,
          syncStopped: false,
        }),
      },
    });

    expect(screen.getByText("Step 3 of 10 · Stage output")).toBeInTheDocument();
    const rows = [...container.querySelectorAll(".phase")];
    expect(rows[0]).toHaveClass("done");
    expect(rows[0].querySelector(".phase-time")).toHaveTextContent("0.1 s");
    expect(rows[1].querySelector(".phase-time")).toHaveTextContent("0.2 s");
    expect(rows[2]).toHaveClass("active");
    expect(rows[2]).toHaveAttribute("aria-current", "step");
    expect(rows[2].querySelector(".phase-time")).toBeEmptyDOMElement();
    expect(rows[3].querySelector(".phase-time")).toBeEmptyDOMElement();
    expect(
      (
        container.querySelector(".phase-list") as HTMLElement
      ).style.getPropertyValue("--phase-progress"),
    ).toBe(String(2 / 9));
  });

  it("settles on the last step with a measured total when finished", () => {
    const { container } = render(RunRail, {
      props: {
        rail: runRail({
          local: [
            ...local,
            { phase: "validating_output", elapsed_ms: 400 },
            { phase: "backing_up", elapsed_ms: 500 },
            { phase: "committing", elapsed_ms: 700 },
            { phase: "writing_receipt", elapsed_ms: 800 },
            { phase: "finished", elapsed_ms: 900 },
          ],
          sync: [],
          executing: false,
          localFailed: false,
          syncing: false,
          syncExpected: false,
          syncStopped: false,
        }),
      },
    });

    expect(screen.getByText("Finished · 0.9 s")).toBeInTheDocument();
    expect(container.querySelector(".phase-list")).toHaveClass("is-finished");
    expect(
      [...container.querySelectorAll(".phase-time")].filter(
        (time) => time.textContent === "Skipped",
      ),
    ).toHaveLength(2);
  });
});

describe("RestorePanel", () => {
  const result = {
    run_id: "run-1",
    plan_id: "plan-1",
    committed_files: 2,
    created_files: 1,
    updated_files: 1,
    deleted_files: 0,
    skipped_operations: 0,
    backup_sha256: "backup-sha256",
    backup_file_count: 2,
    receipt_path: "receipt.json",
  };
  const preview = {
    paths: [
      {
        path: "account/filament/safe.json",
        action: "create" as const,
        safe_to_restore: true,
        current_sha256: "committed",
        expected_committed_sha256: "committed",
      },
      {
        path: "account/filament/changed.json",
        action: "update" as const,
        safe_to_restore: false,
        current_sha256: "external",
        expected_committed_sha256: "committed",
      },
    ],
  };

  it("shows exact safe and externally changed journal-owned paths", () => {
    render(RestorePanel, {
      props: { result, preview, onPreview: vi.fn(), onRestore: vi.fn() },
    });

    expect(
      screen.getByText("1 owned path can be restored"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("1 externally changed path will be preserved"),
    ).toBeInTheDocument();
    expect(screen.getByText("account/filament/safe.json")).toBeInTheDocument();
    expect(screen.getByText("Ready to restore")).toBeInTheDocument();
    expect(
      screen.getByText("Externally changed - preserve"),
    ).toBeInTheDocument();
  });

  it("requires confirmation with the run ID before restoring", async () => {
    const onRestore = vi.fn();
    render(RestorePanel, {
      props: { result, preview, onPreview: vi.fn(), onRestore },
    });

    await fireEvent.click(
      screen.getByRole("button", { name: "Restore 1 owned path" }),
    );
    const dialog = screen.getByRole("dialog", {
      name: "Restore files from run run-1?",
    });
    expect(onRestore).not.toHaveBeenCalled();
    expect(dialog).toHaveTextContent(
      "Only 1 unchanged journal-owned path will be restored.",
    );
    const confirm = within(dialog).getByRole("button", {
      name: "Restore owned files",
    });
    expect(confirm).toHaveClass("danger-button");
    await fireEvent.click(confirm);
    expect(onRestore).toHaveBeenCalledOnce();
  });

  it("reports the exact rollback outcome", () => {
    render(RestorePanel, {
      props: {
        result,
        preview,
        rollback: { rolled_back: 1, external_conflicts: 1, failed: 0 },
        onPreview: vi.fn(),
        onRestore: vi.fn(),
      },
    });

    expect(screen.getByRole("status")).toHaveTextContent(
      "Restored 1; preserved 1 externally changed; 0 failed.",
    );
  });
});
