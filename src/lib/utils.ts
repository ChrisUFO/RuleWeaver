import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import type { ToolEntry } from "@/types/rule";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function getToolFileName(tool: ToolEntry): string {
  return tool.paths.localPathTemplate.split(/[/\\]/).pop() || "";
}
