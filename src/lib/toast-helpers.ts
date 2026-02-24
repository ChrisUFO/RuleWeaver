import type { useToast } from "@/components/ui/toast";

type ToastFunction = ReturnType<typeof useToast>["addToast"];

type ToastVariant = "success" | "error" | "warning" | "info";

interface ToastOptions {
  title: string;
  description: string;
  duration?: number;
}

function createToast(addToast: ToastFunction, variant: ToastVariant) {
  return (options: ToastOptions) => {
    addToast({
      title: options.title,
      description: options.description,
      variant,
      duration: options.duration,
    });
  };
}

export const toast = {
  success: (addToast: ToastFunction, options: ToastOptions) =>
    createToast(addToast, "success")(options),

  error: (addToast: ToastFunction, options: ToastOptions | { title: string; error: unknown }) => {
    const description =
      "error" in options
        ? options.error instanceof Error
          ? options.error.message
          : typeof options.error === "string"
            ? options.error
            : "Unknown error"
        : options.description;
    createToast(addToast, "error")({ ...options, description });
  },

  warning: (addToast: ToastFunction, options: ToastOptions) =>
    createToast(addToast, "warning")(options),

  info: (addToast: ToastFunction, options: ToastOptions) => createToast(addToast, "info")(options),
};

export const toastMessages = {
  saved: (itemType: string, name: string) => ({
    title: `${itemType} Saved`,
    description: `"${name}" has been updated`,
  }),
  created: (itemType: string, name: string) => ({
    title: `${itemType} Created`,
    description: `"${name}" has been created`,
  }),
  deleted: (itemType: string, name: string, onUndo?: () => void) => ({
    title: `${itemType} Deleted`,
    description: `"${name}" has been deleted`,
    action: onUndo ? { label: "Undo", onClick: onUndo } : undefined,
  }),
  duplicated: (itemType: string, name: string) => ({
    title: `${itemType} Duplicated`,
    description: `"${name}" has been duplicated`,
  }),
  saveFailed: (error: unknown) => ({
    title: "Save Failed",
    error,
  }),
  createFailed: (error: unknown) => ({
    title: "Create Failed",
    error,
  }),
  deleteFailed: (error: unknown) => ({
    title: "Delete Failed",
    error,
  }),
  loadFailed: (itemType: string, error: unknown) => ({
    title: `Failed to Load ${itemType}`,
    error,
  }),
  syncComplete: (count: number) => ({
    title: "Sync Complete",
    description: `${count} file${count !== 1 ? "s" : ""} updated`,
  }),
  syncFailed: (error: unknown) => ({
    title: "Sync Failed",
    error,
  }),
};
