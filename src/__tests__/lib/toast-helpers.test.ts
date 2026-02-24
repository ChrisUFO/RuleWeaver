import { describe, it, expect } from "vitest";
import { toast } from "@/lib/toast-helpers";

describe("toast helpers", () => {
  it("creates success toast with correct structure", () => {
    const calls: Array<Record<string, unknown>> = [];
    const mockAddToast = (options: Record<string, unknown>) => {
      calls.push(options);
    };

    toast.success(mockAddToast, { title: "Success", description: "It worked" });

    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      title: "Success",
      description: "It worked",
      variant: "success",
    });
  });

  it("creates error toast with Error instance", () => {
    const calls: Array<Record<string, unknown>> = [];
    const mockAddToast = (options: Record<string, unknown>) => {
      calls.push(options);
    };

    toast.error(mockAddToast, { title: "Failed", error: new Error("Bad thing") });

    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      title: "Failed",
      description: "Bad thing",
      variant: "error",
    });
  });

  it("creates error toast with string error", () => {
    const calls: Array<Record<string, unknown>> = [];
    const mockAddToast = (options: Record<string, unknown>) => {
      calls.push(options);
    };

    toast.error(mockAddToast, { title: "Failed", error: "String error" });

    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      title: "Failed",
      description: "String error",
      variant: "error",
    });
  });

  it("creates error toast with unknown error type", () => {
    const calls: Array<Record<string, unknown>> = [];
    const mockAddToast = (options: Record<string, unknown>) => {
      calls.push(options);
    };

    toast.error(mockAddToast, { title: "Failed", error: { weird: "object" } });

    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      title: "Failed",
      description: "Unknown error",
      variant: "error",
    });
  });

  it("creates error toast with description directly", () => {
    const calls: Array<Record<string, unknown>> = [];
    const mockAddToast = (options: Record<string, unknown>) => {
      calls.push(options);
    };

    toast.error(mockAddToast, { title: "Failed", description: "Direct description" });

    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      title: "Failed",
      description: "Direct description",
      variant: "error",
    });
  });

  it("creates warning toast", () => {
    const calls: Array<Record<string, unknown>> = [];
    const mockAddToast = (options: Record<string, unknown>) => {
      calls.push(options);
    };

    toast.warning(mockAddToast, { title: "Warning", description: "Be careful" });

    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      title: "Warning",
      description: "Be careful",
      variant: "warning",
    });
  });

  it("creates info toast", () => {
    const calls: Array<Record<string, unknown>> = [];
    const mockAddToast = (options: Record<string, unknown>) => {
      calls.push(options);
    };

    toast.info(mockAddToast, { title: "Info", description: "FYI" });

    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      title: "Info",
      description: "FYI",
      variant: "info",
    });
  });
});
