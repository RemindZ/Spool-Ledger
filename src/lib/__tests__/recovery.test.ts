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
import ExecutionPhases from "../components/ExecutionPhases.svelte";
import RestorePanel from "../components/RestorePanel.svelte";
import RunStatus from "../components/RunStatus.svelte";
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
        localPhase: "finished",
        syncPhase: "finished",
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
        localPhase: "finished",
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
        localPhase: "finished",
        syncPhase: "monitoring",
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
        localPhase: "finished",
        syncPhase: "finished",
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
      props: { result: runResult(1), localPhase: "finished" },
    });

    expect(
      screen.getByRole("heading", { name: "1 local file committed" }),
    ).toBeInTheDocument();
    expect(screen.getByText("run-1")).toBeInTheDocument();
  });

  it("describes a running local transaction without actions", () => {
    render(RunStatus, {
      props: { result: null, localPhase: "backing_up" },
    });

    expect(
      screen.getByRole("heading", { name: "Committing local profiles" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});

describe("ExecutionPhases", () => {
  it("summarizes the active step and keeps the full list collapsed", () => {
    const { container } = render(ExecutionPhases, {
      props: {
        localPhase: "staging",
        syncPhase: null,
        syncExpected: true,
      },
    });

    expect(
      screen.getByText("Step 3 of 11: Stage local profiles"),
    ).toBeInTheDocument();
    expect(container.querySelectorAll(".phase-segments > i")).toHaveLength(11);
    expect(
      container.querySelector("details.phase-details"),
    ).not.toHaveAttribute("open");
  });

  it("counts only applicable steps when synchronization is not expected", () => {
    render(ExecutionPhases, {
      props: { localPhase: "finished", syncPhase: null, syncExpected: false },
    });

    expect(screen.getByText("All 8 steps complete")).toBeInTheDocument();
  });

  it("marks only event-backed local phases complete or active", () => {
    render(ExecutionPhases, {
      props: {
        localPhase: "staging",
        syncPhase: null,
        syncExpected: true,
      },
    });

    expect(
      screen.getByText("Validate frozen plan").closest("li"),
    ).toHaveAttribute("data-state", "complete");
    expect(
      screen.getByText("Stage local profiles").closest("li"),
    ).toHaveAttribute("data-state", "active");
    expect(
      screen.getByText("Validate generated output").closest("li"),
    ).toHaveAttribute("data-state", "pending");
  });

  it("projects synchronization events after every local phase", () => {
    render(ExecutionPhases, {
      props: {
        localPhase: "finished",
        syncPhase: "monitoring",
        syncExpected: true,
      },
    });

    expect(
      screen.getByText("Finish local transaction").closest("li"),
    ).toHaveAttribute("data-state", "complete");
    expect(
      screen.getByText("Launch Bambu Studio").closest("li"),
    ).toHaveAttribute("data-state", "complete");
    expect(
      screen.getByText("Observe Bambu evidence").closest("li"),
    ).toHaveAttribute("data-state", "active");
    expect(
      screen.getByText("Finish synchronization").closest("li"),
    ).toHaveAttribute("data-state", "pending");
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
