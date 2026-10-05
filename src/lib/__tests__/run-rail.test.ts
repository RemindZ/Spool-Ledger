import { describe, expect, it } from "vitest";
import {
  RUN_STEPS,
  formatSeconds,
  runRail,
  type RunRailInput,
} from "../run-rail";

function input(overrides: Partial<RunRailInput> = {}): RunRailInput {
  return {
    local: [],
    sync: [],
    executing: false,
    localFailed: false,
    syncing: false,
    syncExpected: true,
    syncStopped: false,
    ...overrides,
  };
}

const localDone = [
  { phase: "validating", elapsed_ms: 0 },
  { phase: "closing_bambu", elapsed_ms: 100 },
  { phase: "staging", elapsed_ms: 400 },
  { phase: "validating_output", elapsed_ms: 900 },
  { phase: "backing_up", elapsed_ms: 1000 },
  { phase: "committing", elapsed_ms: 1600 },
  { phase: "writing_receipt", elapsed_ms: 1700 },
  { phase: "finished", elapsed_ms: 1900 },
] as const;

describe("run rail", () => {
  it("lists the ten transaction steps with their descriptions", () => {
    expect(RUN_STEPS.map((step) => step.name)).toEqual([
      "Validate frozen plan",
      "Close Bambu Studio gracefully",
      "Stage output",
      "Validate output",
      "Create ZIP backup",
      "Commit files",
      "Write journal and receipt",
      "Launch Bambu Studio",
      "Monitor synchronization",
      "Finished or stopped",
    ]);
    expect(RUN_STEPS[1].hint).toBe(
      "Asks Bambu Studio to save and quit. Never force-closes.",
    );
    expect(RUN_STEPS[9].hint).toBe(
      "Local run complete. AMS visibility is confirmed by you.",
    );
  });

  it("is idle before a transaction starts", () => {
    const rail = runRail(input());
    expect(rail.summary).toBe("No transaction started");
    expect(rail.steps.every((step) => step.state === "pending")).toBe(true);
    expect(rail.progress).toBe(0);
    expect(rail.finished).toBe(false);
  });

  it("marks the active step and times only completed steps while running", () => {
    const rail = runRail(
      input({ executing: true, local: localDone.slice(0, 5) }),
    );
    expect(rail.summary).toBe("Step 5 of 10 · Create ZIP backup");
    expect(rail.steps.map((step) => step.state).slice(0, 6)).toEqual([
      "done",
      "done",
      "done",
      "done",
      "active",
      "pending",
    ]);
    expect(rail.steps.map((step) => step.durationMs).slice(0, 5)).toEqual([
      100,
      300,
      500,
      100,
      null,
    ]);
    expect(rail.activeIndex).toBe(4);
  });

  it("moves into synchronization after the local transaction finishes", () => {
    const rail = runRail(
      input({
        local: [...localDone],
        syncing: true,
        sync: [
          { phase: "launching", elapsed_ms: 0 },
          { phase: "monitoring", elapsed_ms: 800 },
        ],
      }),
    );
    expect(rail.steps[6]).toMatchObject({ state: "done", durationMs: 200 });
    expect(rail.steps[7]).toMatchObject({ state: "done", durationMs: 800 });
    expect(rail.steps[8].state).toBe("active");
    expect(rail.summary).toBe("Step 9 of 10 · Monitor synchronization");
  });

  it("settles on a measured total when synchronization finishes", () => {
    const rail = runRail(
      input({
        local: [...localDone],
        sync: [
          { phase: "launching", elapsed_ms: 0 },
          { phase: "monitoring", elapsed_ms: 800 },
          { phase: "finished", elapsed_ms: 3800 },
        ],
      }),
    );
    expect(rail.finished).toBe(true);
    expect(rail.steps[8]).toMatchObject({ state: "done", durationMs: 3000 });
    expect(rail.steps[9].state).toBe("active");
    expect(rail.summary).toBe("Finished · 5.7 s");
    expect(rail.progress).toBe(1);
  });

  it("skips the synchronization steps when nothing was committed", () => {
    const rail = runRail(input({ local: [...localDone], syncExpected: false }));
    expect(rail.steps[7].state).toBe("skipped");
    expect(rail.steps[8].state).toBe("skipped");
    expect(rail.finished).toBe(true);
    expect(rail.summary).toBe("Finished · 1.9 s");
  });

  it("stops on the failed step when the commit fails", () => {
    const rail = runRail(
      input({ localFailed: true, local: localDone.slice(0, 6) }),
    );
    expect(rail.steps[5].state).toBe("failed");
    expect(rail.steps[5].durationMs).toBeNull();
    expect(rail.steps[9].state).toBe("pending");
    expect(rail.stopped).toBe(true);
    expect(rail.summary).toBe("Stopped at step 6 · Commit files");
  });

  it("keeps the committed run and marks monitoring stopped when it is cancelled", () => {
    const rail = runRail(
      input({
        local: [...localDone],
        syncStopped: true,
        sync: [
          { phase: "launching", elapsed_ms: 0 },
          { phase: "monitoring", elapsed_ms: 500 },
        ],
      }),
    );
    expect(rail.steps[6].state).toBe("done");
    expect(rail.steps[8].state).toBe("stopped");
    expect(rail.steps[8].durationMs).toBeNull();
    expect(rail.steps[9].state).toBe("active");
    expect(rail.stopped).toBe(true);
    expect(rail.finished).toBe(true);
    expect(rail.summary).toBe("Stopped · 2.4 s");
  });

  it("formats durations in seconds with one decimal", () => {
    expect(formatSeconds(0)).toBe("0.0 s");
    expect(formatSeconds(4730)).toBe("4.7 s");
  });
});
