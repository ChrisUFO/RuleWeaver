import type { SyncProgressEvent } from "@/types/rule";

export interface SyncProgressState {
  currentFile: string;
  currentFileIndex: number;
  totalFiles: number;
  completedFiles: { path: string; success: boolean }[];
}

export const EMPTY_SYNC_PROGRESS_STATE: SyncProgressState = {
  currentFile: "",
  currentFileIndex: 0,
  totalFiles: 0,
  completedFiles: [],
};

function appendCompletedFile(
  completedFiles: { path: string; success: boolean }[],
  path: string,
  success: boolean
) {
  const last = completedFiles[completedFiles.length - 1];
  if (last && last.path === path && last.success === success) {
    return completedFiles;
  }

  const existingIndex = completedFiles.findIndex((file) => file.path === path);
  if (existingIndex >= 0) {
    if (completedFiles[existingIndex].success === success) {
      return completedFiles;
    }

    const next = [...completedFiles];
    next[existingIndex] = { path, success };
    return next;
  }

  return [...completedFiles, { path, success }];
}

export function applySyncProgressEvent(
  previous: SyncProgressState,
  event: SyncProgressEvent
): SyncProgressState {
  const totalFiles = Number.isFinite(event.totalFiles) ? event.totalFiles : previous.totalFiles;
  const currentFileIndex = Number.isFinite(event.currentFileIndex)
    ? event.currentFileIndex
    : previous.currentFileIndex;

  if (event.phase === "start") {
    return {
      currentFile: "",
      currentFileIndex: 0,
      totalFiles,
      completedFiles: [],
    };
  }

  if (event.phase === "progress") {
    const currentFile = event.currentFile ?? previous.currentFile;
    const completedFiles =
      event.currentFile && typeof event.itemSuccess === "boolean"
        ? appendCompletedFile(previous.completedFiles, event.currentFile, event.itemSuccess)
        : previous.completedFiles;

    return {
      currentFile,
      currentFileIndex,
      totalFiles,
      completedFiles,
    };
  }

  if (event.phase === "complete") {
    return {
      ...previous,
      currentFile: event.currentFile ?? "",
      currentFileIndex,
      totalFiles,
    };
  }

  if (event.phase === "error") {
    return {
      ...previous,
      currentFile: event.currentFile ?? previous.currentFile,
      currentFileIndex,
      totalFiles,
    };
  }

  return previous;
}
