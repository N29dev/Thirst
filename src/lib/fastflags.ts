// Roblox Player client config, not Studio. Roblox's September 2025
// allowlist can change; entries outside it are ignored by the Player.
export type FlagType = "boolean" | "integer";
export type FlagGroup = "Graphics" | "Geometry" | "Display";

export type FlagDefinition = {
  name: string;
  label: string;
  group: FlagGroup;
  type: FlagType;
  description: string;
};

export const PLAYER_FLAGS: FlagDefinition[] = [
  {name:"DFFlagTextureQualityOverrideEnabled",label:"Manual texture quality",group:"Graphics",type:"boolean",description:"Enables the texture quality override."},
  {name:"DFIntTextureQualityOverride",label:"Texture quality level",group:"Graphics",type:"integer",description:"Controls texture resolution when the override is enabled. Roblox may change supported values."},
  {name:"FIntDebugForceMSAASamples",label:"Anti-aliasing (MSAA)",group:"Graphics",type:"integer",description:"Requested MSAA sample count, such as 0, 2 or 4. Supported values depend on the client and GPU."},
  {name:"DFIntDebugFRMQualityLevelOverride",label:"Graphics quality override",group:"Graphics",type:"integer",description:"Requests a graphics quality level; client behavior may vary."},
  {name:"FIntFRMMaxGrassDistance",label:"Maximum grass distance",group:"Graphics",type:"integer",description:"Requested maximum grass rendering distance."},
  {name:"FIntFRMMinGrassDistance",label:"Minimum grass distance",group:"Graphics",type:"integer",description:"Requested minimum grass rendering distance."},
  {name:"FIntGrassMovementReducedMotionFactor",label:"Grass motion factor",group:"Graphics",type:"integer",description:"Reduces grass movement. The client determines the actual effect."},
  {name:"DFIntCSGLevelOfDetailSwitchingDistance",label:"Mesh detail distance",group:"Geometry",type:"integer",description:"CSG mesh level-of-detail switching distance."},
  {name:"DFIntCSGLevelOfDetailSwitchingDistanceL12",label:"Mesh LOD 1→2 distance",group:"Geometry",type:"integer",description:"CSG mesh detail switching distance."},
  {name:"DFIntCSGLevelOfDetailSwitchingDistanceL23",label:"Mesh LOD 2→3 distance",group:"Geometry",type:"integer",description:"CSG mesh detail switching distance."},
  {name:"DFIntCSGLevelOfDetailSwitchingDistanceL34",label:"Mesh LOD 3→4 distance",group:"Geometry",type:"integer",description:"CSG mesh detail switching distance."},
  {name:"DFFlagDisableDPIScale",label:"Disable DPI scaling",group:"Display",type:"boolean",description:"Requests that Roblox skip DPI scaling. High-DPI displays may behave differently."},
  {name:"FFlagHandleAltEnterFullscreenManually",label:"Alt+Enter fullscreen",group:"Display",type:"boolean",description:"Uses the alternate fullscreen shortcut implementation."},
  {name:"FFlagDebugGraphicsPreferD3D11",label:"Prefer Direct3D 11 (Windows)",group:"Display",type:"boolean",description:"Select Direct3D 11 where supported. Not useful on macOS."},
  {name:"FFlagDebugGraphicsPreferVulkan",label:"Prefer Vulkan",group:"Display",type:"boolean",description:"Select Vulkan where supported. Not every device supports it."},
  {name:"FFlagDebugGraphicsPreferOpenGL",label:"Prefer OpenGL",group:"Display",type:"boolean",description:"Select OpenGL where supported; may conflict with other backend choices."},
  {name:"FFlagDebugSkyGray",label:"Gray debug sky",group:"Display",type:"boolean",description:"Changes sky rendering for supported client versions."},
  {name:"DFFlagDebugPauseVoxelizer",label:"Pause voxelizer",group:"Display",type:"boolean",description:"Debug rendering toggle; may negatively affect the game."}
];
const DEFINITIONS = new Map(PLAYER_FLAGS.map(flag => [flag.name,flag]));

export type FFlagStatus = {
  path: string;
  platform: string;
  playerDetected: boolean;
  fileExists: boolean;
  flags: Record<string,string>;
  ignoredKeys: string[];
  note: string;
};

export type FFlagPreset = "low" | "balanced" | "quality";
export const FFLAG_PRESETS: Record<FFlagPreset,{label:string;description:string;flags:Record<string,string>}> = {
  low: {
    label: "Lower graphics",
    description: "Lower texture quality and MSAA. Aims to reduce GPU load; results vary.",
    flags: {
      DFFlagTextureQualityOverrideEnabled:"True",
      DFIntTextureQualityOverride:"1",
      FIntDebugForceMSAASamples:"0"
    }
  },
  balanced: {
    label: "Balanced",
    description: "Middle texture quality and modest anti-aliasing.",
    flags: {
      DFFlagTextureQualityOverrideEnabled:"True",
      DFIntTextureQualityOverride:"2",
      FIntDebugForceMSAASamples:"2"
    }
  },
  quality: {
    label: "Higher quality",
    description: "Higher texture quality and anti-aliasing, potentially more GPU usage.",
    flags: {
      DFFlagTextureQualityOverrideEnabled:"True",
      DFIntTextureQualityOverride:"3",
      FIntDebugForceMSAASamples:"4"
    }
  }
};

export function flagDefinition(name: string): FlagDefinition | undefined {
  return DEFINITIONS.get(name);
}

export function cleanFlagValue(name:string, input:unknown):string {
  const def = flagDefinition(name);
  if(!def) throw new Error(name+" is not on Roblox's published Player allowlist.");
  if (def.type === "boolean") {
    const raw = String(input).trim().toLowerCase();
    if (raw === "true" || raw === "false") return raw === "true" ? "True" : "False";
    throw new Error(name+" must be True or False.");
  }
  if (typeof input !== "number" && typeof input !== "string") {
    throw new Error(name+" must be an integer.");
  }
  const raw = String(input).trim();
  if (!/^-?\d{1,11}$/.test(raw)) throw new Error(name+" must be a signed 32-bit integer.");
  const n = Number(raw);
  if (!Number.isSafeInteger(n) || n < -2147483648 || n > 2147483647) {
    throw new Error(name+" is outside the 32-bit integer range.");
  }
  return String(n);
}

export type FFlagImportResult = {
  flags: Record<string, string>;
  disabled: Record<string, string>;
  ignored: string[];
  invalid: string[];
  format: "roblox" | "preset";
  name: string | null;
};

type NamedPresetFlag = { flag: unknown; value: unknown; enabled?: unknown };

// Accept Roblox ClientAppSettings.json as well as the named-presets format
// used by launchers such as Bloxstrap ("name", "flags" array, "type", "games").
// Only allowlisted Player flags are imported. Never import game overrides,
// commands, account data or unsupported flags from someone else's preset.
export function importFFlagsJson(raw: string): FFlagImportResult {
  if (raw.length > 64_000) throw new Error("JSON is too large (64 KB maximum).");
  let parsed: unknown;
  try { parsed = JSON.parse(raw); }
  catch { throw new Error("Invalid JSON. Choose a Roblox settings file or a named Fast Flags preset."); }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("Fast Flags JSON must be an object.");
  }

  const document = parsed as Record<string, unknown>;
  const isPreset = Array.isArray(document.flags);
  if ("flags" in document && !isPreset) {
    throw new Error("Preset 'flags' must be an array of {flag, value, enabled} objects.");
  }

  const flags: Record<string, string> = {};
  const disabled: Record<string, string> = {};
  const ignored: string[] = [];
  const invalid: string[] = [];
  const result: FFlagImportResult = {
    flags, disabled, ignored, invalid,
    format: isPreset ? "preset" : "roblox",
    name: isPreset && typeof document.name === "string"
      ? document.name.trim().slice(0, 70) || null : null
  };

  const add = (name: string, value: unknown, enabled: boolean) => {
    if (!flagDefinition(name)) { ignored.push(name); return; }
    try {
      const cleaned = cleanFlagValue(name, value);
      // An explicitly disabled value must NOT become active on import.
      if (enabled) {
        flags[name] = cleaned;
        delete disabled[name];
      } else {
        disabled[name] = cleaned;
        delete flags[name];
      }
    } catch {
      invalid.push(name);
    }
  };

  if (isPreset) {
    if ((document.flags as unknown[]).length > 512) {
      throw new Error("Preset has too many entries (512 maximum).");
    }
    for (const item of document.flags as unknown[]) {
      if (!item || typeof item !== "object" || Array.isArray(item)) {
        invalid.push("Invalid entry"); continue;
      }
      const entry = item as NamedPresetFlag;
      if (typeof entry.flag !== "string") {
        invalid.push("Missing flag name"); continue;
      }
      if (entry.enabled !== undefined && typeof entry.enabled !== "boolean") {
        invalid.push(entry.flag); continue;
      }
      add(entry.flag, entry.value, entry.enabled !== false);
    }
  } else {
    for (const [name, value] of Object.entries(document)) add(name, value, true);
  }

  if (!Object.keys(flags).length && !Object.keys(disabled).length) {
    throw new Error("No supported Roblox Player flags were found in this JSON.");
  }
  return result;
}

export function exportFFlagsJson(flags: Record<string, string>): string {
  const data: Record<string, string> = {};
  for (const name of Object.keys(flags).sort()) {
    data[name] = cleanFlagValue(name, flags[name]);
  }
  return JSON.stringify(data, null, 2);
}

export function exportNamedFFlagsJson(
  enabledFlags: Record<string, string>,
  disabledFlags: Record<string, string> = {},
  name = "Thirst preset"
): string {
  const cleanName = name.trim().slice(0, 70) || "Thirst preset";
  const keys = [...new Set([
    ...Object.keys(enabledFlags), ...Object.keys(disabledFlags)
  ])].sort();
  const flags = keys.map(flag => ({
    flag,
    value: cleanFlagValue(flag,
      Object.prototype.hasOwnProperty.call(enabledFlags, flag)
        ? enabledFlags[flag] : disabledFlags[flag]),
    enabled: Object.prototype.hasOwnProperty.call(enabledFlags, flag)
  }));
  return JSON.stringify({
    name: cleanName,
    flags,
    type: "default",
    games: []
  }, null, 2);
}
