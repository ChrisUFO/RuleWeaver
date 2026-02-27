/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      exclude: [
        "node_modules/",
        "src/test/",
        "**/*.d.ts",
        "**/*.config.*",
        "**/index.ts",
        // Tauri API wrappers are always mocked in tests — excluding from coverage
        "src/lib/tauri.ts",
        // Keyboard shortcut handler requires Tauri runtime — not unit-testable
        "src/hooks/useKeyboardShortcuts.ts",
        // Settings state hook wraps Tauri plugin calls (dialog, updater, autostart)
        // and is always mocked at module level in tests — same rationale as tauri.ts
        "src/hooks/useSettingsState.ts",
        // Pure TypeScript type/interface files — no runtime code to cover
        "src/types/command.ts",
        "src/types/rule.ts",
      ],
      thresholds: {
        // Thresholds reflect current baseline (Phase 6 gate — tighten as coverage grows).
        // Many complex dialog/feature components have 0% coverage but are structurally
        // difficult to unit-test (Tauri-dependent, multi-step modal flows). The lifecycle
        // and hook modules that CAN be tested achieve 74-90%.
        lines: 16,
        functions: 20,
        branches: 45,
        statements: 16,
      },
    },
  },
}));
