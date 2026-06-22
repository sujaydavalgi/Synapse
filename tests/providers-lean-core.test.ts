import { describe, it, expect } from "vitest";
import {
  bootstrapDefaultProviders,
  MODULE_CLASSIFICATIONS,
  ModuleOwnership,
  OFFICIAL_PACKAGE_NAMES,
} from "../src/providers/index.js";

describe("lean-core providers (TypeScript mirror)", () => {
  it("lists official package names", () => {
    expect(OFFICIAL_PACKAGE_NAMES).toContain("spanda-gps");
    expect(OFFICIAL_PACKAGE_NAMES).toContain("spanda-ros2");
  });

  it("classifies core and shim modules", () => {
    expect(
      MODULE_CLASSIFICATIONS.some(
        (m) => m.module === "providers" && m.ownership === ModuleOwnership.Core,
      ),
    ).toBe(true);
    expect(
      MODULE_CLASSIFICATIONS.some(
        (m) =>
          m.module === "transport_mqtt" &&
          m.ownership === ModuleOwnership.Deprecated,
      ),
    ).toBe(true);
  });

  it("bootstraps default transport shims", () => {
    const registry = bootstrapDefaultProviders();
    const ids = registry.listTransports();
    expect(ids.some((id) => id.package === "spanda-mqtt")).toBe(true);
    expect(ids.some((id) => id.package === "spanda-ros2")).toBe(true);
  });
});
