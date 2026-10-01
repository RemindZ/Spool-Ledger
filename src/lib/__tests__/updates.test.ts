// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import UpdateNotice from "../components/UpdateNotice.svelte";
import { rememberSkippedVersion, shouldNotifyUpdate } from "../updates";
import type { AvailableUpdate } from "../types";

afterEach(cleanup);
beforeEach(() => localStorage.clear());

const available = (tag: string): AvailableUpdate => ({
  status: "available",
  current: "0.9.0",
  latest: tag.slice(1),
  tag,
  url: `https://github.com/RemindZ/Spool-Ledger/releases/tag/${tag}`,
});

describe("update notice", () => {
  it("names both versions and offers update, not now, and skip", async () => {
    const onUpdate = vi.fn();
    const onDismiss = vi.fn();
    const onSkip = vi.fn();
    render(UpdateNotice, {
      props: { update: available("v0.10.0"), onUpdate, onDismiss, onSkip },
    });

    expect(
      screen.getByRole("heading", { name: "Update available" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Spool Ledger 0.10.0 is available. You have 0.9.0."),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Update opens the release page in your browser."),
    ).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Update" }));
    await fireEvent.click(screen.getByRole("button", { name: "Not now" }));
    await fireEvent.click(
      screen.getByRole("button", { name: "Skip this version" }),
    );
    expect(onUpdate).toHaveBeenCalledOnce();
    expect(onDismiss).toHaveBeenCalledOnce();
    expect(onSkip).toHaveBeenCalledOnce();
  });
});

describe("skipped versions", () => {
  it("suppresses only the skipped release and shows newer ones", () => {
    expect(shouldNotifyUpdate(available("v0.10.0"), localStorage)).toBe(true);
    rememberSkippedVersion(localStorage, "v0.10.0");
    expect(shouldNotifyUpdate(available("v0.10.0"), localStorage)).toBe(false);
    expect(shouldNotifyUpdate(available("v0.11.0"), localStorage)).toBe(true);
  });

  it("never notifies when the build is current", () => {
    expect(
      shouldNotifyUpdate({ status: "current", current: "0.9.0" }, localStorage),
    ).toBe(false);
  });

  it("still notifies when storage is unavailable", () => {
    const blocked = {
      getItem: () => {
        throw new Error("blocked");
      },
      setItem: () => {
        throw new Error("blocked");
      },
    } as unknown as Storage;
    expect(() => rememberSkippedVersion(blocked, "v0.10.0")).not.toThrow();
    expect(shouldNotifyUpdate(available("v0.10.0"), blocked)).toBe(true);
  });
});

describe("update notice failures", () => {
  it("keeps the notice open with a readable error when the browser cannot open", () => {
    render(UpdateNotice, {
      props: {
        update: available("v0.10.0"),
        error: "Could not open the release page. Please try again.",
        onUpdate: vi.fn(),
        onDismiss: vi.fn(),
        onSkip: vi.fn(),
      },
    });

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Could not open the release page. Please try again.",
    );
    expect(screen.getByRole("button", { name: "Update" })).toBeEnabled();
  });
});
