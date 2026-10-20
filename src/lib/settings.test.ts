import { describe, expect, it } from "vitest";
import { audioOptions, cleanName, DEFAULT_SETTINGS, mergeSettings, pixelFixOptions } from "./settings";

describe("mergeSettings", () => {
  it("returns the defaults for nothing saved", () => {
    expect(mergeSettings(null)).toEqual(DEFAULT_SETTINGS);
    expect(mergeSettings("broken")).toEqual(DEFAULT_SETTINGS);
  });

  it("defaults audio to RMS at -14 dBFS", () => {
    expect(DEFAULT_SETTINGS.audio.mode).toBe("rms");
    expect(DEFAULT_SETTINGS.audio.targetRms).toBe(-14);
  });

  it("keeps valid saved values and repairs invalid ones", () => {
    const merged = mergeSettings({
      lastTab: "audio-convert",
      pixelFix: { recursive: true, output: "prefix", prefix: "fx/_", view: "grid" },
      audio: { mode: "peak", targetRms: -16, targetPeak: 12, format: "exe", suffix: "_x/y" },
    });
    expect(merged.lastTab).toBe("audio-convert");
    expect(merged.pixelFix).toMatchObject({ recursive: true, output: "prefix", prefix: "fx_", view: "grid" });
    expect(merged.audio).toMatchObject({ mode: "peak", targetRms: -16, targetPeak: 0, format: "keep", suffix: "_xy" });
  });

  it("reads settings saved by the first version", () => {
    const merged = mergeSettings({
      lastTab: "audio-normalize",
      audio: { convertEnabled: true, convertFormat: "ogg", mode: "rms" },
    });
    expect(merged.lastTab).toBe("audio-convert");
    expect(merged.audio.format).toBe("ogg");
    expect(mergeSettings({ audio: { convertEnabled: false, convertFormat: "mp3" } }).audio.format).toBe("keep");
  });
});

describe("tool options", () => {
  it("only converts when a format is picked", () => {
    expect(audioOptions(DEFAULT_SETTINGS.audio).convert).toBeNull();
    expect(audioOptions({ ...DEFAULT_SETTINGS.audio, format: "mp3" }).convert).toBe("mp3");
  });

  it("leaves the view out of the Pixel Fix options", () => {
    expect(pixelFixOptions(DEFAULT_SETTINGS.pixelFix)).toEqual({
      recursive: false,
      output: "replace",
      folder: "fixed",
      prefix: "fixed_",
    });
  });
});

describe("cleanName", () => {
  it("drops characters Windows doesn't allow in file names", () => {
    expect(cleanName('_n<>:"|?*')).toBe("_n");
  });
});
