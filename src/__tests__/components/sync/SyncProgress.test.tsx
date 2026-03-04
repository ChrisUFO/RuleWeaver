import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { SyncProgress } from "@/components/sync/SyncProgress";

describe("SyncProgress", () => {
  it("does not render while idle", () => {
    render(
      <SyncProgress
        isSyncing={false}
        currentFile=""
        currentFileIndex={0}
        totalFiles={0}
        completedFiles={[]}
      />
    );

    expect(screen.queryByText(/syncing artifacts/i)).not.toBeInTheDocument();
  });

  it("renders progress details and extracts file name from windows paths", () => {
    render(
      <SyncProgress
        isSyncing
        currentFile="C:\\Users\\chris\\.config\\opencode\\rules\\my-rule.md"
        currentFileIndex={2}
        totalFiles={3}
        completedFiles={[
          { path: "C:\\Users\\chris\\.config\\opencode\\rules\\first-rule.md", success: true },
        ]}
      />
    );

    expect(screen.getByText(/2 of 3 files/i)).toBeInTheDocument();
    expect(screen.getByText("my-rule.md")).toBeInTheDocument();
    expect(screen.getByText("first-rule.md")).toBeInTheDocument();
  });

  it("shows fallback copy when current step details are unavailable", () => {
    render(
      <SyncProgress
        isSyncing
        currentFile=""
        currentFileIndex={0}
        totalFiles={0}
        completedFiles={[]}
      />
    );

    expect(screen.getByText(/waiting for sync step details/i)).toBeInTheDocument();
    expect(screen.getByText(/preparing file list/i)).toBeInTheDocument();
  });
});
