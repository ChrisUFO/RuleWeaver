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

// Context used to pass generated ids from DialogContent down to DialogTitle/DialogDescription
const DialogIdContext = React.createContext<{ titleId: string; descriptionId: string } | null>(
  null
);

const DialogContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { onClose?: () => void }
>(({ className, children, onClose, ...props }, forwardedRef) => {
  const accessibilityEnabled = featureManager.isEnabled(FEATURE_FLAGS.DIALOG_ACCESSIBILITY);
  const innerRef = React.useRef<HTMLDivElement>(null);

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

  useFocusTrap(innerRef, accessibilityEnabled);

  // Escape key handler
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
        "aria-describedby": descriptionId,
      }
    : {};

  return (
    <DialogIdContext.Provider value={{ titleId, descriptionId }}>
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
        <button
          className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none"
          onClick={onClose}
        >
          <X className="h-4 w-4" />
          <span className="sr-only">Close</span>
        </button>
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
