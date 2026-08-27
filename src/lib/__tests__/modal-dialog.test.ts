// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import ModalDialogHarness from "./fixtures/ModalDialogHarness.svelte";

beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function showModal() {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function close() {
    this.removeAttribute("open");
  };
});

afterEach(cleanup);

describe("ModalDialog", () => {
  it("opens as a native dialog and focuses Cancel first", async () => {
    render(ModalDialogHarness);
    await fireEvent.click(
      screen.getByRole("button", { name: "Open confirmation" }),
    );

    expect(screen.getByRole("dialog")).toHaveAttribute("open");
    expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus();
    expect(
      screen.getByText(
        "Cloud persistence and AMS visibility are not guaranteed.",
      ),
    ).toBeInTheDocument();
  });

  it("closes on native cancel and restores focus to the trigger", async () => {
    render(ModalDialogHarness);
    const trigger = screen.getByRole("button", { name: "Open confirmation" });
    trigger.focus();
    await fireEvent.click(trigger);

    await fireEvent(
      screen.getByRole("dialog"),
      new Event("cancel", { cancelable: true }),
    );

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  it("closes when the backdrop itself is clicked", async () => {
    render(ModalDialogHarness);
    const trigger = screen.getByRole("button", { name: "Open confirmation" });
    await fireEvent.click(trigger);
    await fireEvent.click(screen.getByRole("dialog"));

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("runs the explicit confirmation action", async () => {
    render(ModalDialogHarness);
    await fireEvent.click(
      screen.getByRole("button", { name: "Open confirmation" }),
    );
    await fireEvent.click(
      screen.getByRole("button", { name: "Commit migration" }),
    );

    expect(screen.getByLabelText("Confirmation result")).toHaveTextContent(
      "confirmed",
    );
  });
});
