import { describe, expect, it } from "vitest";
import { groupFiles } from "./groups";

describe("groupFiles", () => {
  it("makes one group per folder and one for loose files", () => {
    const groups = groupFiles(
      ["D:\\ui", "D:\\other\\logo.png"],
      ["D:\\ui\\coin.png", "D:\\ui\\icons\\gem.png", "D:\\other\\logo.png"],
    );
    expect(groups.map((g) => [g.kind, g.name, g.files.length])).toEqual([
      ["folder", "ui", 2],
      ["files", "Files", 1],
    ]);
    expect(groups[0].files[1]).toEqual({ path: "D:\\ui\\icons\\gem.png", name: "gem.png", subfolder: "icons" });
  });

  it("keeps empty folders and puts shared files in the first folder", () => {
    const groups = groupFiles(["D:\\ui", "D:\\ui\\icons", "D:\\empty"], ["D:\\ui\\icons\\gem.png"]);
    expect(groups.map((g) => g.files.length)).toEqual([1, 0, 0]);
  });

  it("matches paths regardless of case and slashes", () => {
    const groups = groupFiles(["d:/UI/"], ["D:\\ui\\coin.png"]);
    expect(groups[0].files).toHaveLength(1);
  });
});
