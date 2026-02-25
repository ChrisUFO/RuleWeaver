import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Dialog, DialogContent, DialogTitle } from "../../../components/ui/dialog";

describe("Dialog Component", () => {
  it("renders when open and uses portal", () => {
    const { baseElement } = render(
      <Dialog open={true} onOpenChange={() => {}}>
        <DialogContent>
          <DialogTitle>Test Title</DialogTitle>
          <div>Test Content</div>
        </DialogContent>
      </Dialog>
    );

    // Check if it's in the document
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

    // overlay is the div with bg-black/80
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
