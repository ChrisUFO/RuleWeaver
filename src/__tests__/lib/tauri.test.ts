import { describe, it, expect, vi, beforeEach } from "vitest";
import { api } from "@/lib/tauri";
import type { Rule, AdapterType } from "@/types/rule";

vi.mock("@/lib/tauri", () => ({
  api: {
    rules: {
      getAll: vi.fn(),
      getById: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
      toggle: vi.fn(),
    },
    storage: {
      exportConfiguration: vi.fn(),
      importConfiguration: vi.fn(),
      previewImport: vi.fn(),
    },
    mcp: {
      getStatus: vi.fn(),
      start: vi.fn(),
      stop: vi.fn(),
      restart: vi.fn(),
    },
  },
}));

describe("api.rules", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("create", () => {
    it("should create a rule with correct input shape", async () => {
      const mockRule: Rule = {
        id: "123",
        name: "Test Rule",
        content: "Test content",
        scope: "global",
        targetPaths: null,
        enabledAdapters: ["gemini", "opencode"] as AdapterType[],
        enabled: true,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      const input = {
        name: "Test Rule",
        content: "Test content",
        scope: "global" as const,
        enabledAdapters: ["gemini", "opencode"] as AdapterType[],
      };

      vi.mocked(api.rules.create).mockResolvedValue(mockRule);

      const result = await api.rules.create(input);

      expect(api.rules.create).toHaveBeenCalledWith(input);
      expect(result).toEqual(mockRule);
    });

    it("should handle rule creation with local scope and target paths", async () => {
      const mockRule: Rule = {
        id: "456",
        name: "Local Rule",
        content: "Local content",
        scope: "local",
        targetPaths: ["/path/to/repo"],
        enabledAdapters: ["cline"] as AdapterType[],
        enabled: true,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      const input = {
        name: "Local Rule",
        content: "Local content",
        scope: "local" as const,
        targetPaths: ["/path/to/repo"],
        enabledAdapters: ["cline"] as AdapterType[],
      };

      vi.mocked(api.rules.create).mockResolvedValue(mockRule);

      const result = await api.rules.create(input);

      expect(result.scope).toBe("local");
      expect(result.targetPaths).toEqual(["/path/to/repo"]);
    });
  });

  describe("update", () => {
    it("should update a rule with partial input", async () => {
      const mockRule: Rule = {
        id: "123",
        name: "Updated Rule",
        content: "Original content",
        scope: "global",
        targetPaths: null,
        enabledAdapters: ["gemini"] as AdapterType[],
        enabled: true,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      vi.mocked(api.rules.update).mockResolvedValue(mockRule);

      const result = await api.rules.update("123", { name: "Updated Rule" });

      expect(api.rules.update).toHaveBeenCalledWith("123", { name: "Updated Rule" });
      expect(result.name).toBe("Updated Rule");
    });

    it("should handle update with empty adapters array", async () => {
      const mockRule: Rule = {
        id: "123",
        name: "Test",
        content: "Content",
        scope: "global",
        targetPaths: null,
        enabledAdapters: [] as AdapterType[],
        enabled: false,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      vi.mocked(api.rules.update).mockResolvedValue(mockRule);

      const result = await api.rules.update("123", { enabledAdapters: [] });

      expect(result.enabledAdapters).toEqual([]);
    });
  });

  describe("toggle", () => {
    it("should toggle rule enabled state", async () => {
      const mockRule: Rule = {
        id: "123",
        name: "Test",
        content: "Content",
        scope: "global",
        targetPaths: null,
        enabledAdapters: ["gemini"] as AdapterType[],
        enabled: false,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      vi.mocked(api.rules.toggle).mockResolvedValue(mockRule);

      const result = await api.rules.toggle("123", false);

      expect(api.rules.toggle).toHaveBeenCalledWith("123", false);
      expect(result.enabled).toBe(false);
    });
  });

  describe("delete", () => {
    it("should delete a rule", async () => {
      vi.mocked(api.rules.delete).mockResolvedValue(undefined);

      await api.rules.delete("123");

      expect(api.rules.delete).toHaveBeenCalledWith("123");
    });
  });
});

describe("api.storage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("exportConfiguration", () => {
    it("should export configuration to specified path", async () => {
      vi.mocked(api.storage.exportConfiguration).mockResolvedValue(undefined);

      await api.storage.exportConfiguration("/path/to/export.json");

      expect(api.storage.exportConfiguration).toHaveBeenCalledWith("/path/to/export.json");
    });

    it("should handle YAML export", async () => {
      vi.mocked(api.storage.exportConfiguration).mockResolvedValue(undefined);

      await api.storage.exportConfiguration("/path/to/export.yaml");

      expect(api.storage.exportConfiguration).toHaveBeenCalledWith("/path/to/export.yaml");
    });
  });

  describe("importConfiguration", () => {
    it("should import configuration from specified path", async () => {
      vi.mocked(api.storage.importConfiguration).mockResolvedValue(undefined);

      await api.storage.importConfiguration("/path/to/import.json", "overwrite");

      expect(api.storage.importConfiguration).toHaveBeenCalledWith(
        "/path/to/import.json",
        "overwrite"
      );
    });

    it("should import with skip mode for existing items", async () => {
      vi.mocked(api.storage.importConfiguration).mockResolvedValue(undefined);

      await api.storage.importConfiguration("/path/to/import.json", "skip");

      expect(api.storage.importConfiguration).toHaveBeenCalledWith("/path/to/import.json", "skip");
    });
  });

  describe("previewImport", () => {
    it("should return parsed configuration preview", async () => {
      const preview = {
        version: "1.0",
        exported_at: new Date().toISOString(),
        rules: [] as Rule[],
        commands: [],
        skills: [],
      };

      vi.mocked(api.storage.previewImport).mockResolvedValue(preview);

      const result = await api.storage.previewImport("/path/to/import.json");

      expect(result.version).toBe("1.0");
      expect(result.rules).toEqual([]);
    });

    it("should reject unsupported version", async () => {
      vi.mocked(api.storage.previewImport).mockRejectedValue(
        new Error("Unsupported configuration version: 2.0. Only 1.0 is supported.")
      );

      await expect(api.storage.previewImport("/path/to/import.json")).rejects.toThrow(
        "Unsupported configuration version"
      );
    });
  });
});

describe("api.mcp", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("server control", () => {
    it("should start MCP server", async () => {
      vi.mocked(api.mcp.start).mockResolvedValue(undefined);

      await api.mcp.start();

      expect(api.mcp.start).toHaveBeenCalled();
    });

    it("should stop MCP server", async () => {
      vi.mocked(api.mcp.stop).mockResolvedValue(undefined);

      await api.mcp.stop();

      expect(api.mcp.stop).toHaveBeenCalled();
    });

    it("should restart MCP server", async () => {
      vi.mocked(api.mcp.restart).mockResolvedValue(undefined);

      await api.mcp.restart();

      expect(api.mcp.restart).toHaveBeenCalled();
    });

    it("should get MCP status", async () => {
      const status = { running: true, port: 3000 };
      vi.mocked(api.mcp.getStatus).mockResolvedValue(status);

      const result = await api.mcp.getStatus();

      expect(result).toEqual(status);
      expect(result.running).toBe(true);
      expect(result.port).toBe(3000);
    });

    it("should handle stopped MCP server status", async () => {
      const status = { running: false, port: null };
      vi.mocked(api.mcp.getStatus).mockResolvedValue(status);

      const result = await api.mcp.getStatus();

      expect(result.running).toBe(false);
      expect(result.port).toBeNull();
    });
  });
});
