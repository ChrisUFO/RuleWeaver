import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import type { ToolEntry } from "@/types/rule";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function getToolFileName(tool: ToolEntry): string {
  return tool.paths.localPathTemplate.split(/[/\\]/).pop() || "";
}

export const WORKSPACE_ROOT_VAR = "${WORKSPACE_ROOT}";

/**
 * Normalizes a path for visual previews by ensuring consistent forward slashes
 * and trimming leading/trailing whitespace.
 */
export function normalizePath(path: string): string {
  return path.trim().replace(/\\/g, "/");
}

/**
 * Resolves a path for visual preview based on a base path and workspace variables.
 */
export function resolveWorkspacePathPreview(path: string, basePath?: string | null): string {
  if (!basePath || !path) return path;

  if (path.startsWith("./")) {
    const relative = path.substring(2);
    const normalizedBase =
      basePath.endsWith("/") || basePath.endsWith("\\") ? basePath.slice(0, -1) : basePath;
    return normalizePath(`${normalizedBase}/${relative}`);
  }

  if (path.includes(WORKSPACE_ROOT_VAR)) {
    return normalizePath(path.replace(WORKSPACE_ROOT_VAR, basePath));
  }

  return path;
}

/**
 * Generates a name for a duplicated item with "(Copy) N" suffixing.
 * e.g. "Rule" -> "Rule (Copy)"
 * e.g. "Rule (Copy)" -> "Rule (Copy) 2"
 * e.g. "Rule (Copy) 2" -> "Rule (Copy) 3"
 */
export function generateDuplicateName(name: string, existingNames: string[]): string {
  const copySuffix = " (Copy)";
  const MAX_NAME_LENGTH = 100;
  const MAX_ITERATIONS = 10000;

  let baseName = name;
  let counter = 1;

  // Improved regex to handle multiple (Copy) or already suffixed copies
  // It looks for the LAST occurrence of (Copy) followed by optional number
  const copyMatch = name.match(/(.*) \(Copy\)(?: (\d+))?$/);
  if (copyMatch) {
    baseName = copyMatch[1];
    if (copyMatch[2]) {
      counter = parseInt(copyMatch[2], 10) + 1;
    } else {
      counter = 2;
    }
  }

  let newName = counter === 1 ? `${baseName}${copySuffix}` : `${baseName}${copySuffix} ${counter}`;

  // Ensure it doesn't exceed length limit while being unique
  let iterations = 0;
  while (
    (existingNames.includes(newName) || newName.length > MAX_NAME_LENGTH) &&
    iterations < MAX_ITERATIONS
  ) {
    iterations++;
    if (newName.length > MAX_NAME_LENGTH) {
      // If too long, truncate baseName more
      baseName = baseName.substring(0, baseName.length - 5);
    } else {
      counter++;
    }
    newName = counter === 1 ? `${baseName}${copySuffix}` : `${baseName}${copySuffix} ${counter}`;

    // Safety break
    if (baseName.length < 5 && newName.length > MAX_NAME_LENGTH) break;
  }

  return newName;
}
