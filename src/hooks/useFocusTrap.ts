import { useEffect, useRef } from "react";

const FOCUSABLE_SELECTORS = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
  "details > summary",
].join(", ");

export function useFocusTrap(containerRef: React.RefObject<HTMLElement | null>, enabled: boolean) {
  const returnFocusRef = useRef<Element | null>(null);

  useEffect(() => {
    if (!enabled || !containerRef.current) return;

    // Capture the element that had focus before the dialog opened
    returnFocusRef.current = document.activeElement;

    // Focus the first focusable element inside the container
    const focusable = containerRef.current.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTORS);
    if (focusable.length > 0) {
      focusable[0].focus();
    } else {
      // If no focusable children, focus the container itself
      containerRef.current.focus();
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Tab" || !containerRef.current) return;

      const focusableEls = Array.from(
        containerRef.current.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTORS)
      );
      if (focusableEls.length === 0) return;

      const first = focusableEls[0];
      const last = focusableEls[focusableEls.length - 1];

      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      // Restore focus to the element that had it before the dialog opened
      if (returnFocusRef.current && "focus" in returnFocusRef.current) {
        (returnFocusRef.current as HTMLElement).focus();
      }
    };
  }, [enabled, containerRef]);
}
