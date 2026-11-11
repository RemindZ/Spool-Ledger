import type { ExecutionPhase, SyncPhase } from "./types";

export interface PhaseMark<T extends string> {
  phase: T;
  /** Monotonic milliseconds from the backend when the phase began. */
  elapsed_ms: number;
}

export type RailStepState =
  "pending" | "active" | "done" | "skipped" | "failed" | "stopped";

export interface RailStep {
  name: string;
  hint: string;
  state: RailStepState;
  /** Measured from backend phase events, only for completed steps. */
  durationMs: number | null;
}

export interface RunRail {
  steps: RailStep[];
  summary: string;
  /** Index of the highlighted step, or -1 before a transaction starts. */
  activeIndex: number;
  /** Share of the track filled up to the highlighted step, 0 to 1. */
  progress: number;
  finished: boolean;
  stopped: boolean;
}

export interface RunRailInput {
  local: ReadonlyArray<PhaseMark<ExecutionPhase>>;
  sync: ReadonlyArray<PhaseMark<SyncPhase>>;
  executing: boolean;
  localFailed: boolean;
  syncing: boolean;
  syncExpected: boolean;
  /** Synchronization failed, timed out, or was cancelled after the commit. */
  syncStopped: boolean;
}

export const RUN_STEPS: ReadonlyArray<{ name: string; hint: string }> = [
  {
    name: "Validate frozen plan",
    hint: "Re-checks every operation against the selected account.",
  },
  {
    name: "Close Bambu Studio gracefully",
    hint: "Asks Bambu Studio to save and quit. Never force-closes.",
  },
  {
    name: "Stage output",
    hint: "Writes every file to a private transaction folder first.",
  },
  {
    name: "Validate output",
    hint: "Parses each staged preset before anything is committed.",
  },
  {
    name: "Create ZIP backup",
    hint: "Backs up every file that will change, with a checksum.",
  },
  {
    name: "Commit files",
    hint: "Moves the reviewed files into the Bambu account.",
  },
  {
    name: "Write journal and receipt",
    hint: "Records exactly what changed so it can be restored.",
  },
  {
    name: "Launch Bambu Studio",
    hint: "Reopens Bambu Studio so it loads the new presets.",
  },
  {
    name: "Monitor synchronization",
    hint: "Watches for cloud setting IDs to appear.",
  },
  {
    name: "Finished or stopped",
    hint: "Local run complete. AMS visibility is confirmed by you.",
  },
];

const LOCAL_PHASES: ExecutionPhase[] = [
  "validating",
  "closing_bambu",
  "staging",
  "validating_output",
  "backing_up",
  "committing",
  "writing_receipt",
];
const SYNC_PHASES: SyncPhase[] = ["launching", "monitoring"];
const FINAL = RUN_STEPS.length - 1;

export function formatSeconds(milliseconds: number): string {
  return `${(milliseconds / 1000).toFixed(1)} s`;
}

/**
 * Step states and durations for a sequence of phases. A phase's duration runs
 * from its own start to the next reported start, or to the end mark.
 */
function measure<T extends string>(
  order: readonly T[],
  marks: ReadonlyArray<PhaseMark<T>>,
  end: number | null,
): Array<{ started: boolean; durationMs: number | null }> {
  const starts = new Map(marks.map((mark) => [mark.phase, mark.elapsed_ms]));
  return order.map((phase, index) => {
    const start = starts.get(phase);
    if (start === undefined) return { started: false, durationMs: null };
    const next = order
      .slice(index + 1)
      .map((later) => starts.get(later))
      .find((value) => value !== undefined);
    const stop = next ?? end;
    return {
      started: true,
      durationMs: stop === null ? null : Math.max(0, stop - start),
    };
  });
}

export function runRail(input: RunRailInput): RunRail {
  const localEnd =
    input.local.find((mark) => mark.phase === "finished")?.elapsed_ms ?? null;
  const syncEnd =
    input.sync.find((mark) => mark.phase === "finished")?.elapsed_ms ?? null;
  const local = measure(LOCAL_PHASES, input.local, localEnd);
  const sync = measure(SYNC_PHASES, input.sync, syncEnd);
  const measured = [...local, ...sync];
  const lastStarted = measured.reduce(
    (last, step, index) => (step.started ? index : last),
    -1,
  );
  const localFinished = localEnd !== null;
  const syncFinished = syncEnd !== null;

  const steps: RailStep[] = RUN_STEPS.map((step) => ({
    ...step,
    state: "pending",
    durationMs: null,
  }));
  let activeIndex = -1;
  let finished = false;
  let stopped = false;

  measured.forEach((step, index) => {
    if (step.durationMs !== null) {
      steps[index].state = "done";
      steps[index].durationMs = step.durationMs;
    } else if (!step.started && index < lastStarted) {
      steps[index].state = "skipped";
    }
  });

  if (input.localFailed && !localFinished) {
    if (lastStarted >= 0) {
      steps[lastStarted].state = "failed";
      steps[lastStarted].durationMs = null;
    }
    activeIndex = lastStarted;
    stopped = true;
  } else if (!localFinished) {
    if (lastStarted >= 0) {
      steps[lastStarted].state = "active";
      activeIndex = lastStarted;
    }
  } else if (!input.syncExpected) {
    steps[7].state = "skipped";
    steps[8].state = "skipped";
    activeIndex = FINAL;
    finished = true;
  } else if (syncFinished) {
    activeIndex = FINAL;
    finished = true;
  } else if (input.syncStopped) {
    const reached = Math.max(7, lastStarted);
    steps[reached].state = "stopped";
    steps[reached].durationMs = null;
    activeIndex = FINAL;
    finished = true;
    stopped = true;
  } else {
    activeIndex = Math.max(7, lastStarted);
    steps[activeIndex].state = "active";
    steps[activeIndex].durationMs = null;
  }
  if (finished) steps[FINAL].state = "active";

  const total = steps.reduce((sum, step) => sum + (step.durationMs ?? 0), 0);
  const summary =
    activeIndex < 0
      ? "No transaction started"
      : input.localFailed && !localFinished
        ? `Stopped at step ${activeIndex + 1} · ${RUN_STEPS[activeIndex].name}`
        : finished
          ? `${stopped ? "Stopped" : "Finished"} · ${formatSeconds(total)}`
          : `Step ${activeIndex + 1} of ${RUN_STEPS.length} · ${RUN_STEPS[activeIndex].name}`;

  return {
    steps,
    summary,
    activeIndex,
    progress: activeIndex < 0 ? 0 : activeIndex / FINAL,
    finished,
    stopped,
  };
}
