import { describe, it, expect } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";

describe("extern bridge syntax", () => {
  it("parses extern python fn", () => {
    const source = `extern python fn py_version() -> Int;`;
    const program = parse(tokenize(source));
    expect(program.externFunctions[0]?.bridge).toBe("python");
    expect(program.externFunctions[0]?.library).toBe("python");
  });

  it("parses extern cpp fn", () => {
    const source = `extern cpp fn ros_publish(path: String) -> Int;`;
    const program = parse(tokenize(source));
    expect(program.externFunctions[0]?.bridge).toBe("cpp");
    expect(program.externFunctions[0]?.name).toBe("ros_publish");
  });
});
