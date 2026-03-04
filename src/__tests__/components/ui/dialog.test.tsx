import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { featureManager } from "../../../lib/featureManager";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
} from "../../../components/ui/dialog";

vi.mock("../../../lib/featureManager", () => ({
  featureManager: { isEnabled: vi.fn(() => true) },
  FEATURE_FLAGS: {
    DIALOG_ACCESSIBILITY: "dialog_accessibility",
    THEME_PERSISTENCE: "theme_persistence",
    ENHANCED_ERROR_UX: "enhanced_error_ux",
    NATIVE_SKILL_SYNC: "native_skill_sync",
    UNIFIED_ARTIFACT_STATUS: "unified_artifact_status",
    EXECUTION_REDACTION: "execution_redaction",
  },
}));

// ---------------------------------------------------------------------------
// 1. Core rendering (flag enabled by default)
// ---------------------------------------------------------------------------
describe("Dialog Component — core rendering", () => {
  beforeEach(() => {
    vi.mocked(featureManager.isEnabled).mockReturnValue(true);
  });

  it("renders when open and uses portal", () => {
    const { baseElement } = render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent>
          <DialogTitle>Test Title</DialogTitle>
          <div>Test Content</div>
        </DialogContent>
      </Dialog>
    );

    expect(screen.getByText("Test Title")).toBeInTheDocument();

    // Portal check: The dialog should be a child of document.body, not the container
    const content = screen.getByText("Test Content").closest(".fixed.inset-0.z-50");
    expect(content?.parentElement).toBe(baseElement);
  });

  it("calls onOpenChange when overlay is clicked", () => {
    const onOpenChange = vi.fn();
    render(
      <Dialog open={true} onOpenChange={onOpenChange}>
        <DialogContent>Content</DialogContent>
      </Dialog>
    );

    const overlay = document.body.querySelector(".bg-black\\/80");
    expect(overlay).toBeDefined();
    if (overlay) {
      fireEvent.click(overlay);
      expect(onOpenChange).toHaveBeenCalledWith(false);
    }
  });

  it("calls onClose when close button is clicked", () => {
    const onClose = vi.fn();
    render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent onClose={onClose}>Content</DialogContent>
      </Dialog>
    );

    const closeBtn = screen.getByRole("button", { name: /close/i });
    fireEvent.click(closeBtn);
    expect(onClose).toHaveBeenCalled();
  });

  it("does not render when closed", () => {
    render(
      <Dialog open={false} onOpenChange={() => {}}>
        <DialogContent>Test Content</DialogContent>
      </Dialog>
    );

    expect(screen.queryByText("Test Content")).not.toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// 2. Accessibility (flag enabled)
// ---------------------------------------------------------------------------
describe("Dialog Component — accessibility (flag enabled)", () => {
  beforeEach(() => {
    vi.mocked(featureManager.isEnabled).mockReturnValue(true);
  });

  it("has correct ARIA attributes on DialogContent", () => {
    render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent onClose={() => {}}>
          <DialogTitle>Accessible Title</DialogTitle>
          <DialogDescription>Accessible description</DialogDescription>
        </DialogContent>
      </Dialog>
    );

    const dialog = screen.getByRole("dialog");
    expect(dialog).toBeInTheDocument();
    expect(dialog).toHaveAttribute("aria-modal", "true");

    const titleId = screen.getByText("Accessible Title").getAttribute("id");
    const descId = screen.getByText("Accessible description").getAttribute("id");
    expect(titleId).toBeTruthy();
    expect(descId).toBeTruthy();
    expect(dialog).toHaveAttribute("aria-labelledby", titleId);
    expect(dialog).toHaveAttribute("aria-describedby", descId);
  });

  it("calls onClose when Escape is pressed on a dismissible dialog", () => {
    const onClose = vi.fn();
    render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent onClose={onClose}>
          <DialogTitle>Title</DialogTitle>
        </DialogContent>
      </Dialog>
    );

    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("does NOT call onClose when Escape is pressed on a non-dismissible dialog (no onClose)", () => {
    const onOpenChange = vi.fn();
    render(
      <Dialog open={true} onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogTitle>Non-dismissible</DialogTitle>
        </DialogContent>
      </Dialog>
    );

    fireEvent.keyDown(document, { key: "Escape" });
    // onClose is undefined — neither onOpenChange nor any handler should fire
    expect(onOpenChange).not.toHaveBeenCalled();
  });

  it("traps Tab focus within the dialog", () => {
    render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent onClose={() => {}}>
          <DialogTitle>Focus Trap Test</DialogTitle>
          <button>First</button>
          <button>Second</button>
        </DialogContent>
      </Dialog>
    );

    const buttons = screen.getAllByRole("button");
    // The close button is included. Last interactive element.
    const last = buttons[buttons.length - 1];
    last.focus();

    // Tab from last should cycle back to first
    fireEvent.keyDown(document, { key: "Tab", shiftKey: false });
    // Focus should wrap to the first button
    expect(document.activeElement).toBe(buttons[0]);
  });

  it("traps Shift+Tab focus within the dialog", () => {
    render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent onClose={() => {}}>
          <DialogTitle>Focus Trap Test</DialogTitle>
          <button>First Btn</button>
          <button>Second Btn</button>
        </DialogContent>
      </Dialog>
    );

    const firstBtn = screen.getByText("First Btn");
    firstBtn.focus();
    expect(document.activeElement).toBe(firstBtn);

    // Shift+Tab from first should cycle to last
    fireEvent.keyDown(document, { key: "Tab", shiftKey: true });
    const closeButton = screen.getByRole("button", { name: /close/i });
    expect(document.activeElement).toBe(closeButton);
  });

  it("restores focus to the trigger element after dialog closes", () => {
    const trigger = document.createElement("button");
    trigger.textContent = "Open Dialog";
    document.body.appendChild(trigger);
    trigger.focus();

    const { unmount } = render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent onClose={() => {}}>
          <DialogTitle>Title</DialogTitle>
          <button>Inside</button>
        </DialogContent>
      </Dialog>
    );

    // Unmounting simulates dialog close
    unmount();
    expect(document.activeElement).toBe(trigger);

    document.body.removeChild(trigger);
  });
});

// ---------------------------------------------------------------------------
// 3. Accessibility (flag disabled)
// ---------------------------------------------------------------------------
describe("Dialog Component — accessibility (flag disabled)", () => {
  beforeEach(() => {
    vi.mocked(featureManager.isEnabled).mockReturnValue(false);
  });

  it("does not add ARIA attributes when flag is off", () => {
    render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent onClose={() => {}}>
          <DialogTitle>Title</DialogTitle>
        </DialogContent>
      </Dialog>
    );

    // role="dialog" should not be present when accessibility flag is off
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("does not close on Escape when flag is off", () => {
    const onClose = vi.fn();
    render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent onClose={onClose}>
          <DialogTitle>Title</DialogTitle>
        </DialogContent>
      </Dialog>
    );

    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).not.toHaveBeenCalled();
  });
});
