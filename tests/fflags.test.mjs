import test from "node:test";
import assert from "node:assert/strict";
import {
  importFFlagsJson, exportFFlagsJson, exportNamedFFlagsJson
} from "../src/lib/fastflags.ts";

test("imports the named preset format, keeping disabled flags disabled", () => {
  const input = {
    name: "GK profile",
    type: "default",
    games: [],
    flags: [
      { flag: "DFIntTextureQualityOverride", value: "1", enabled: true },
      { flag: "DFFlagDisableDPIScale", value: "True", enabled: false },
      { flag: "FIntDebugForceMSAASamples", value: "1", enabled: true }
    ]
  };
  const result = importFFlagsJson(JSON.stringify(input));
  assert.equal(result.format, "preset");
  assert.equal(result.name, "GK profile");
  assert.deepEqual(result.flags, {
    DFIntTextureQualityOverride: "1",
    FIntDebugForceMSAASamples: "1"
  });
  assert.deepEqual(result.disabled, { DFFlagDisableDPIScale: "True" });
  assert.deepEqual(result.ignored, []);
});

test("named preset exports round-trip without enabling disabled flags", () => {
  const json = exportNamedFFlagsJson(
    { DFIntTextureQualityOverride: "2", FFlagDebugSkyGray: "False" },
    { DFFlagDisableDPIScale: "True" }, "Example"
  );
  const parsed = JSON.parse(json);
  assert.equal(parsed.name, "Example");
  assert.deepEqual(parsed.games, []);
  assert.equal(parsed.flags.find(item => item.flag === "DFFlagDisableDPIScale").enabled, false);
  const imported = importFFlagsJson(json);
  assert.equal(imported.flags.DFFlagDisableDPIScale, undefined);
  assert.equal(imported.disabled.DFFlagDisableDPIScale, "True");
  assert.equal(imported.flags.DFIntTextureQualityOverride, "2");
});

test("Roblox native settings export includes only enabled flags", () => {
  const json = exportFFlagsJson({ FFlagDebugSkyGray: "true", DFIntTextureQualityOverride: "1" });
  assert.deepEqual(JSON.parse(json), {
    DFIntTextureQualityOverride: "1", FFlagDebugSkyGray: "True"
  });
  const imported = importFFlagsJson(json);
  assert.equal(imported.format, "roblox");
  assert.equal(Object.keys(imported.disabled).length, 0);
});

test("unsupported, invalid and malicious-looking settings are not activated", () => {
  const preset = {
    name: "Imported",
    flags: [
      { flag: "FFlagDebugSkyGray", value: "True", enabled: false },
      { flag: "DFIntTaskSchedulerTargetFps", value: "999", enabled: true },
      { flag: "__proto__", value: "something", enabled: true },
      { flag: "FIntFRMMaxGrassDistance", value: "bad-value", enabled: true },
      { flag: "DFFlagTextureQualityOverrideEnabled", value: "True", enabled: true }
    ],
    games: [{ id: 123, flags: [{ flag: "FFlagDebugSkyGray", value: "True" }] }]
  };
  const imported = importFFlagsJson(JSON.stringify(preset));
  assert.deepEqual(imported.flags, { DFFlagTextureQualityOverrideEnabled: "True" });
  assert.equal(imported.disabled.FFlagDebugSkyGray, "True");
  assert.ok(imported.ignored.includes("DFIntTaskSchedulerTargetFps"));
  assert.ok(imported.ignored.includes("__proto__"));
  assert.ok(imported.invalid.includes("FIntFRMMaxGrassDistance"));
});

test("rejects malformed presets and empty inputs", () => {
  assert.throws(() => importFFlagsJson("[]"), /object/);
  assert.throws(() => importFFlagsJson('{"flags":{}}'), /array/);
  assert.throws(() => importFFlagsJson('{"flags":[]}'), /No supported/);
});
