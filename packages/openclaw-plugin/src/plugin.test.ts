import { describe, expect, it, vi } from "vitest";
import type { OpenClawPluginApi } from "openclaw/plugin-sdk/core";
import plugin from "./plugin.js";

function createTestPluginApi(
  overrides: Partial<OpenClawPluginApi> = {}
): OpenClawPluginApi {
  return {
    id: "actionbook",
    name: "Actionbook",
    description: "Actionbook",
    source: "test",
    config: {} as never,
    pluginConfig: {},
    runtime: {} as never,
    logger: {
      debug: vi.fn(),
      info: vi.fn(),
      warn: vi.fn(),
      error: vi.fn(),
    },
    registerTool: vi.fn(),
    registerHook: vi.fn(),
    registerHttpRoute: vi.fn(),
    registerChannel: vi.fn(),
    registerGatewayMethod: vi.fn(),
    registerCli: vi.fn(),
    registerService: vi.fn(),
    registerProvider: vi.fn(),
    registerCommand: vi.fn(),
    registerContextEngine: vi.fn(),
    resolvePath(input: string) {
      return input;
    },
    on: vi.fn(),
    ...overrides,
  };
}

describe("plugin registration", () => {
  it("registers tools and prompt guidance without info log noise", async () => {
    const registerTool = vi.fn();
    const on = vi.fn();
    const logger = {
      debug: vi.fn(),
      info: vi.fn(),
      warn: vi.fn(),
      error: vi.fn(),
    };

    plugin.register(
      createTestPluginApi({
        registerTool,
        on,
        logger,
      })
    );

    expect(logger.info).not.toHaveBeenCalled();
    expect(on).toHaveBeenCalledTimes(1);
    expect(on.mock.calls[0]?.[0]).toBe("before_prompt_build");

    const beforePromptBuild = on.mock.calls[0]?.[1];
    const result = await beforePromptBuild?.({}, {});
    expect(result).toMatchObject({
      prependSystemContext: expect.stringContaining("Actionbook Browser Automation"),
    });

    expect(registerTool).toHaveBeenCalledTimes(2);
    expect(registerTool.mock.calls.map(([tool]) => tool.name)).toEqual([
      "search_actions",
      "get_action_by_area_id",
    ]);
    expect(registerTool.mock.calls.every(([, opts]) => opts === undefined)).toBe(
      true
    );
  });

  it("validates apiUrl at registration time", () => {
    expect(() =>
      plugin.register(
        createTestPluginApi({
          pluginConfig: { apiUrl: "not-a-url" },
        })
      )
    ).toThrow('actionbook: invalid apiUrl "not-a-url"');
  });
});
