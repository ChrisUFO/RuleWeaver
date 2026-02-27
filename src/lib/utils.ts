import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import type { ToolEntry } from "@/types/rule";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function getToolFileName(tool: ToolEntry): string {
  return tool.paths.localPathTemplate.split(/[/\\]/).pop() || "";
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

  let baseName = name;
  let counter = 1;

  // Check if name already has a (Copy) suffix
  const copyMatch = name.match(/ \(Copy\)(?: (\d+))?$/);
  if (copyMatch) {
    baseName = name.substring(0, copyMatch.index);
    if (copyMatch[1]) {
      counter = parseInt(copyMatch[1], 10) + 1;
    } else {
      counter = 2;
    }
  }

  let newName = counter === 1 ? `${baseName}${copySuffix}` : `${baseName}${copySuffix} ${counter}`;

  // Ensure it doesn't exceed length limit while being unique
  while (existingNames.includes(newName) || newName.length > MAX_NAME_LENGTH) {
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
