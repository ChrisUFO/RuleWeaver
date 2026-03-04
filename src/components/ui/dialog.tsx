import * as React from "react";
import { createPortal } from "react-dom";

import { cn } from "@/lib/utils";
import { X } from "lucide-react";
import { useFocusTrap } from "@/hooks/useFocusTrap";
import { featureManager, FEATURE_FLAGS } from "@/lib/featureManager";

interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: React.ReactNode;
}

const Dialog: React.FC<DialogProps> = ({ open, onOpenChange, children }) => {
  if (!open) return null;

  return createPortal(
    <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
      <div
        className="fixed inset-0 bg-black/80 pointer-events-auto"
        onClick={() => onOpenChange(false)}
      />
      <div className="relative z-50 pointer-events-auto">{children}</div>
    </div>,
    document.body
  );
};

// Context propagates generated ids AND tracks whether a DialogDescription is present,
// so aria-describedby is only set when a description element actually renders.
interface DialogIdContextValue {
  titleId: string;
  descriptionId: string;
  registerDescription: () => void;
  unregisterDescription: () => void;
}

const DialogIdContext = React.createContext<DialogIdContextValue | null>(null);

const DialogContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { onClose?: () => void }
>(({ className, children, onClose, ...props }, forwardedRef) => {
  const accessibilityEnabled = featureManager.isEnabled(FEATURE_FLAGS.DIALOG_ACCESSIBILITY);
  const innerRef = React.useRef<HTMLDivElement>(null);
  const [hasDescription, setHasDescription] = React.useState(false);

  // Merge the forwarded ref with the local ref so focus-trap can access the DOM node
  const setRef = React.useCallback(
    (node: HTMLDivElement | null) => {
      (innerRef as React.MutableRefObject<HTMLDivElement | null>).current = node;
      if (typeof forwardedRef === "function") {
        forwardedRef(node);
      } else if (forwardedRef) {
        (forwardedRef as React.MutableRefObject<HTMLDivElement | null>).current = node;
      }
    },
    [forwardedRef]
  );

  const titleId = React.useId();
  const descriptionId = React.useId();

  const registerDescription = React.useCallback(() => setHasDescription(true), []);
  const unregisterDescription = React.useCallback(() => setHasDescription(false), []);

  useFocusTrap(innerRef, accessibilityEnabled);

  // Escape key handler — only fires when the flag is on and the dialog is dismissible
  React.useEffect(() => {
    if (!accessibilityEnabled || !onClose) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        onClose();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [accessibilityEnabled, onClose]);

  const ariaProps = accessibilityEnabled
    ? {
        role: "dialog" as const,
        "aria-modal": true,
        "aria-labelledby": titleId,
        // Only include aria-describedby when a DialogDescription actually rendered;
        // a dangling reference confuses screen readers.
        ...(hasDescription && { "aria-describedby": descriptionId }),
        // tabIndex allows the container to receive focus as a fallback
        // when no focusable children exist inside the dialog.
        tabIndex: -1,
      }
    : {};

  return (
    <DialogIdContext.Provider
      value={{ titleId, descriptionId, registerDescription, unregisterDescription }}
    >
      <div
        ref={setRef}
        className={cn(
          "grid w-full max-w-lg gap-4 border bg-background p-6 shadow-lg sm:rounded-lg",
          className
        )}
        {...ariaProps}
        {...props}
      >
        {children}
        {/* Only render the close button when the dialog is dismissible */}
        {onClose && (
          <button
            className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none"
            onClick={onClose}
            aria-label="Close"
          >
            <X className="h-4 w-4" />
            <span className="sr-only">Close</span>
          </button>
        )}
      </div>
    </DialogIdContext.Provider>
  );
});
DialogContent.displayName = "DialogContent";

const DialogHeader = ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
  <div className={cn("flex flex-col space-y-1.5 text-center sm:text-left", className)} {...props} />
);
DialogHeader.displayName = "DialogHeader";

const DialogFooter = ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn("flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2", className)}
    {...props}
  />
);
DialogFooter.displayName = "DialogFooter";

const DialogTitle = React.forwardRef<HTMLHeadingElement, React.HTMLAttributes<HTMLHeadingElement>>(
  ({ className, id, ...props }, ref) => {
    const ctx = React.useContext(DialogIdContext);
    return (
      <h2
        ref={ref}
        id={id ?? ctx?.titleId}
        className={cn("text-lg font-semibold leading-none tracking-tight", className)}
        {...props}
      />
    );
  }
);
DialogTitle.displayName = "DialogTitle";

const DialogDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, id, ...props }, ref) => {
  const ctx = React.useContext(DialogIdContext);

  // Notify DialogContent that a description is present so it can set aria-describedby
  React.useEffect(() => {
    ctx?.registerDescription();
    return () => ctx?.unregisterDescription();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <p
      ref={ref}
      id={id ?? ctx?.descriptionId}
      className={cn("text-sm text-muted-foreground", className)}
      {...props}
    />
  );
});
DialogDescription.displayName = "DialogDescription";

export { Dialog, DialogContent, DialogHeader, DialogFooter, DialogTitle, DialogDescription };
