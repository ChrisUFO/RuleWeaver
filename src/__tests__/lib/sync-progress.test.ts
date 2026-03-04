import { describe, expect, it } from "vitest";
import { applySyncProgressEvent, EMPTY_SYNC_PROGRESS_STATE } from "@/lib/sync-progress";

describe("sync progress mapping", () => {
  it("resets state on start events", () => {
    const next = applySyncProgressEvent(
      {
        currentFile: "a.md",
        currentFileIndex: 2,
        totalFiles: 3,
        completedFiles: [{ path: "a.md", success: true }],
      },
      {
        phase: "start",
        currentFileIndex: 0,
        totalFiles: 5,
      }
    );

    expect(next).toEqual({
      ...EMPTY_SYNC_PROGRESS_STATE,
      totalFiles: 5,
    });
  });

  it("tracks per-file progress and completion results", () => {
    const next = applySyncProgressEvent(EMPTY_SYNC_PROGRESS_STATE, {
      phase: "progress",
      currentFile: "/tmp/rules.md",
      currentFileIndex: 1,
      totalFiles: 2,
      itemSuccess: true,
    });

    expect(next.currentFile).toBe("/tmp/rules.md");
    expect(next.currentFileIndex).toBe(1);
    expect(next.totalFiles).toBe(2);
    expect(next.completedFiles).toEqual([{ path: "/tmp/rules.md", success: true }]);
  });

  it("keeps previous state when detail fields are unavailable", () => {
    const previous = {
      currentFile: "/tmp/a.md",
      currentFileIndex: 1,
      totalFiles: 2,
      completedFiles: [{ path: "/tmp/a.md", success: true }],
    };

    const next = applySyncProgressEvent(previous, {
      phase: "progress",
      currentFileIndex: 1,
      totalFiles: 2,
    });

    expect(next.currentFile).toBe(previous.currentFile);
    expect(next.completedFiles).toEqual(previous.completedFiles);
  });
});
