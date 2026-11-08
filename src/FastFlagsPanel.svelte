<script lang="ts">
  import { onMount } from "svelte";
  import { readFastFlags, writeFastFlags } from "./lib/backend";
  import {
    PLAYER_FLAGS, FFLAG_PRESETS, cleanFlagValue, exportFFlagsJson,
    flagDefinition, importFFlagsJson
  } from "./lib/fastflags";
  import type { FFlagStatus, FFlagPreset } from "./lib/fastflags";

  let status: FFlagStatus | null = null;
  let draft: Record<string,string> = {};
  let saved: Record<string,string> = {};
  let isBusy = false;
  let error = "";
  let info = "";
  let search = "";
  let jsonInput = "";
  let newFlagName = "";
  const presetNames:FFlagPreset[] = ["low","balanced","quality"];

  $: activeKeys = Object.keys(draft).sort();
  $: activeDefinitions = PLAYER_FLAGS.filter(flag =>
    Object.prototype.hasOwnProperty.call(draft,flag.name) &&
    (flag.label.toLowerCase().includes(search.toLowerCase()) ||
      flag.name.toLowerCase().includes(search.toLowerCase()))
  );
  $: unusedDefinitions = PLAYER_FLAGS.filter(flag =>
    !Object.prototype.hasOwnProperty.call(draft,flag.name));
  $: dirty = JSON.stringify(Object.entries(draft).sort()) !== JSON.stringify(Object.entries(saved).sort());

  onMount(() => { void reload(); });

  async function reload() {
    if (isBusy) return;
    isBusy = true;
    error = "";
    try {
      const result = await readFastFlags();
      status = result;
      draft = {...result.flags};
      saved = {...result.flags};
      info = result.fileExists
        ? "Loaded your current Roblox Player settings."
        : "No settings file found yet. Apply a preset or add a flag to create one.";
    } catch (reason) {
      error = String(reason);
    } finally {
      isBusy = false;
    }
  }

  async function save() {
    if (isBusy) return;
    isBusy = true;
    error = "";
    try {
      const canonical:Record<string,string> = {};
      for (const [name,value] of Object.entries(draft)) {
        canonical[name] = cleanFlagValue(name,value);
      }
      const result = await writeFastFlags(canonical);
      status = result;
      draft = {...result.flags};
      saved = {...result.flags};
      info = "Saved to Roblox Player settings. Fully restart Roblox to apply the supported flags.";
    } catch (reason) {
      error = String(reason);
    } finally {
      isBusy = false;
    }
  }

  function applyPreset(name:FFlagPreset) {
    const preset = FFLAG_PRESETS[name];
    draft = {...draft,...preset.flags};
    info = preset.label + " preset loaded. Review the values, then click Save to Roblox.";
    error = "";
  }

  function addFlag() {
    const name = unusedDefinitions.some(flag => flag.name === newFlagName)
      ? newFlagName : unusedDefinitions[0]?.name;
    if(!name) return;
    const def = flagDefinition(name);
    if (!def) return;
    draft = {...draft,[name]:def.type === "boolean" ? "True" : "0"};
    info = "Added " + name + " to your draft.";
    newFlagName = "";
    search = "";
    error = "";
  }

  function removeFlag(name:string) {
    const next = {...draft};
    delete next[name];
    draft = next;
    info = "Removed " + name + " from this draft. Save to apply.";
  }

  function setFlag(name:string,value:string) {
    const next = {...draft,[name]:value};
    if (value === "True" && [
      "FFlagDebugGraphicsPreferD3D11",
      "FFlagDebugGraphicsPreferVulkan",
      "FFlagDebugGraphicsPreferOpenGL"
    ].includes(name)) {
      for (const key of ["FFlagDebugGraphicsPreferD3D11",
        "FFlagDebugGraphicsPreferVulkan","FFlagDebugGraphicsPreferOpenGL"]) {
        if (key !== name && Object.prototype.hasOwnProperty.call(next,key)) next[key] = "False";
      }
    }
    draft = next;
  }

  function importJson(text:string) {
    try {
      const result = importFFlagsJson(text);
      const count = Object.keys(result.flags).length;
      if(!count) throw new Error("No Roblox Player allowlisted flags were found in that JSON.");
      draft = {...draft,...result.flags};
      info = "Imported " + count + " supported flags" +
        (result.ignored.length ? "; skipped " + result.ignored.length + " flags that Roblox Player no longer recognizes." : ".") +
        " Review, then Save to Roblox.";
      error = "";
    } catch (reason) {
      error = String(reason);
    }
  }

  async function importFile(event:Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
      if (file.size > 64_000) throw new Error("The JSON file is too large (64 KB max).");
      const text = await file.text();
      importJson(text);
    } catch (reason) {
      error = String(reason);
    } finally {
      input.value = "";
    }
  }

  async function copyJson() {
    try {
      await navigator.clipboard.writeText(exportFFlagsJson(draft));
      info = "Your draft Fast Flags JSON was copied.";
      error = "";
    } catch (reason) {
      error = "Could not copy JSON: " + String(reason);
    }
  }

  function downloadJson() {
    try {
      const content = exportFFlagsJson(draft);
      const blob = new Blob([content],{type:"application/json;charset=utf-8"});
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = "ClientAppSettings.json";
      document.body.appendChild(link);
      link.click();
      link.remove();
      setTimeout(() => URL.revokeObjectURL(url),1000);
      info = "Exported your draft as ClientAppSettings.json.";
      error = "";
    } catch (reason) {
      error = String(reason);
    }
  }

  async function copyPath() {
    if (!status) return;
    try {
      await navigator.clipboard.writeText(status.path);
      info = "Roblox settings path copied.";
    } catch {
      error = "Clipboard access is not available.";
    }
  }
</script>

<div class="page-title">
  <div>
    <span class="eyebrow">ROBLOX PLAYER CONFIGURATION</span>
    <h1>Fast Flags</h1>
    <p>Graphics presets and a practical ClientAppSettings.json editor.</p>
  </div>
  <span class="chip">PLAYER ALLOWLIST · 18 KEYS</span>
</div>

{#if error}<div class="ff-message ff-error" role="alert">{error}</div>{/if}
{#if info}<div class="ff-message" role="status">{info}</div>{/if}

<section class="panel ff-status">
  <div>
    <h3>Installed Roblox settings</h3>
    {#if status}
      <button class="ff-path" title="Copy settings location" onclick={copyPath}>{status.path} <span>⧉</span></button>
      <p>{status.note}</p>
      <div class="ff-tags">
        <span class="chip">{status.platform}</span>
        <span class="chip">{status.fileExists ? "SETTINGS FOUND" : "NOT CREATED"}</span>
        {#if !status.playerDetected}<span class="chip">ROBLOX INSTALL NOT DETECTED</span>{/if}
        <span class="chip">{activeKeys.length} ACTIVE FLAGS</span>
      </div>
    {:else}
      <p>Check the installed Roblox Player settings file on this device.</p>
    {/if}
  </div>
  <button class="btn secondary" disabled={isBusy || !status} onclick={reload}>{isBusy ? "Working…" : "↻ Reload"}</button>
</section>

<div class="ff-layout">
  <section class="panel ff-editor">
    <div class="ff-section-head">
      <div><h3>Flags editor</h3><p>Values in this list are editable until you save them.</p></div>
      <span class="ff-dirty" class:changed={dirty}>{dirty ? "● Unsaved" : "✓ Saved"}</span>
    </div>
    <div class="ff-controls">
      <label for="ff-search">Find a flag
        <input id="ff-search" class="input" placeholder="Search keys or descriptions…" bind:value={search}/>
      </label>
      <div class="ff-add">
        <label for="ff-new">Add allowed flag
          <select id="ff-new" class="input" bind:value={newFlagName} disabled={!unusedDefinitions.length}>
            <option value="">Choose a flag</option>
            {#each unusedDefinitions as option (option.name)}
              <option value={option.name}>{option.label}</option>
            {/each}
          </select>
        </label>
        <button class="btn secondary" disabled={!unusedDefinitions.length} onclick={addFlag}>+ Add</button>
      </div>
    </div>
    <div class="ff-table">
      {#if activeDefinitions.length}
        {#each activeDefinitions as flag (flag.name)}
          <div class="ff-row">
            <div class="ff-row-info">
              <strong>{flag.label}</strong>
              <code>{flag.name}</code>
              <small>{flag.description}</small>
            </div>
            <div class="ff-row-input">
              {#if flag.type === "boolean"}
                <select class="input" aria-label={flag.name} value={draft[flag.name]}
                  onchange={e => setFlag(flag.name,e.currentTarget.value)}>
                  <option value="True">True</option>
                  <option value="False">False</option>
                </select>
              {:else}
                <input type="number" step="1" class="input" aria-label={flag.name}
                  value={draft[flag.name]} oninput={e => setFlag(flag.name,e.currentTarget.value)}/>
              {/if}
              <button class="ff-remove" title={"Remove " + flag.name} aria-label={"Remove " + flag.name}
                onclick={() => removeFlag(flag.name)}>×</button>
            </div>
          </div>
        {/each}
      {:else}
        <div class="ff-empty">{activeKeys.length ? "No active flags match this search." :
          "No custom flags yet. Apply a graphics preset or add an allowlisted key."}</div>
      {/if}
    </div>
    <div class="ff-editor-actions">
      <button class="btn primary" onclick={save} disabled={isBusy || !status || !dirty}>{isBusy ? "Saving…" : "Save to Roblox"}</button>
      <button class="btn secondary" onclick={reload} disabled={isBusy || !status || !dirty}>Discard draft</button>
      <button class="btn secondary" onclick={() => { draft = {}; search = ""; info = "Supported flags cleared from the draft. Save to reset them."; }}
        disabled={!activeKeys.length || isBusy}>Reset supported flags</button>
    </div>
    <p class="ff-note">Only keys from Roblox's public Player allowlist are edited. Existing unrelated entries are preserved, but Roblox may ignore them. A one-time backup is created before your first overwrite. Fully restart Roblox after saving.</p>
  </section>

  <div class="ff-side">
    <section class="panel ff-box">
      <h3>Quick presets</h3>
      <p>These set just three graphics fields and leave your other flags alone.</p>
      {#each presetNames as name (name)}
        <button class="ff-preset" onclick={() => applyPreset(name)}>
          <strong>{FFLAG_PRESETS[name].label}</strong>
          <span>{FFLAG_PRESETS[name].description}</span>
        </button>
      {/each}
    </section>
    <section class="panel ff-box">
      <h3>JSON loader</h3>
      <p>Import a ClientAppSettings.json file or paste its contents. Unsupported keys are skipped, not secretly enabled.</p>
      <input class="ff-file-input" id="ff-file" type="file" accept=".json,application/json" onchange={importFile}/>
      <label for="ff-file" class="btn secondary ff-upload">↑ Import .json file</label>
      <label class="ff-paste-label" for="ff-json">Paste JSON</label>
      <textarea id="ff-json" class="ff-json-input" rows="6" placeholder="Paste ClientAppSettings.json contents…" bind:value={jsonInput}></textarea>
      <div class="ff-actions">
        <button class="btn secondary" disabled={!jsonInput.trim()} onclick={() => importJson(jsonInput)}>Import pasted JSON</button>
        <button class="btn secondary" onclick={copyJson}>Copy JSON</button>
        <button class="btn secondary" onclick={downloadJson}>↓ Export file</button>
      </div>
    </section>
    <section class="panel ff-box">
      <h3>Compatibility</h3>
      <p>Roblox Player ignores flags outside its published local allowlist. This is not an FPS unlocker. Actual effects vary by platform, hardware and Roblox version.</p>
      {#if status?.ignoredKeys.length}
        <p class="ff-note">{status.ignoredKeys.length} unsupported or invalid keys already exist in your installed JSON. Thirst leaves them untouched.</p>
      {/if}
      <a class="ff-source" href="https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569/" target="_blank" rel="noopener noreferrer">Roblox official allowlist ↗</a>
    </section>
  </div>
</div>

<style>
  .ff-message{border:1px solid #2b6c76;background:#132e3a;color:#a2e4ec;font-size:12px;line-height:1.6;padding:12px 16px;border-radius:10px;margin-bottom:14px;overflow-wrap:anywhere}
  .ff-message.ff-error{border-color:#86414b;background:#2c1923;color:#f7b8bc}
  .ff-status{padding:19px 22px;display:flex;justify-content:space-between;align-items:flex-start;gap:15px;margin-bottom:17px}
  .ff-status h3,.ff-box h3,.ff-section-head h3{margin:0 0 8px;font-size:15px}
  .ff-status p,.ff-box p,.ff-section-head p{margin:0 0 10px;color:#93aab9;font-size:11px;line-height:1.7}
  .ff-status .ff-path{display:block;max-width:100%;border:0;padding:2px 0 7px;background:none;color:#8bd4ea;font-size:11px;text-align:left;word-break:break-all;cursor:pointer}
  .ff-status .ff-path:hover{color:#c0f4ff}
  .ff-path span{padding-left:5px}
  .ff-tags{display:flex;gap:6px;flex-wrap:wrap}
  .ff-layout{display:grid;grid-template-columns:minmax(0,1.8fr) minmax(260px,1fr);align-items:start;gap:15px}
  .ff-editor{overflow:hidden}
  .ff-section-head{padding:20px 21px 12px;display:flex;justify-content:space-between;align-items:flex-start;gap:12px}
  .ff-dirty{color:#96adbb;font-size:11px;white-space:nowrap}
  .ff-dirty.changed{color:#ffd098}
  .ff-controls{padding:0 20px 16px;border-bottom:1px solid #293c4a;display:flex;gap:9px;flex-wrap:wrap;align-items:end}
  .ff-controls>label,.ff-add label{flex:1;min-width:150px;display:flex;flex-direction:column;gap:6px;font-size:10px;font-weight:650;color:#a9bece}
  .ff-add{display:flex;align-items:end;gap:7px;flex:1.2;min-width:230px}
  .ff-add button{min-height:41px}
  .ff-table{max-height:600px;overflow:auto}
  .ff-row{display:flex;justify-content:space-between;gap:15px;align-items:center;padding:15px 20px;border-bottom:1px solid #253848}
  .ff-row-info{min-width:0;display:flex;flex-direction:column;gap:5px;overflow-wrap:anywhere}
  .ff-row-info strong{font-size:12px;color:#e0ecf7}
  .ff-row-info code{font-size:10px;color:#71b8d2}
  .ff-row-info small{font-size:10px;color:#879eb0;line-height:1.6}
  .ff-row-input{display:flex;align-items:center;gap:7px;flex:0 0 140px}
  .ff-row-input .input{height:35px;font-size:12px}
  .ff-remove{color:#f0a2a9;border:1px solid #513943;border-radius:7px;background:#261b27;min-width:30px;height:33px;font-size:18px}
  .ff-remove:hover{background:#39232c}
  .ff-empty{padding:35px 20px;text-align:center;font-size:12px;color:#8299ab}
  .ff-editor-actions{display:flex;flex-wrap:wrap;gap:8px;padding:18px 20px 5px}
  .ff-note{padding:0 20px 18px;margin:9px 0 0;font-size:10px;line-height:1.75;color:#7f97a9}
  .ff-side{display:flex;flex-direction:column;gap:15px;min-width:0}
  .ff-box{padding:19px}
  .ff-preset{width:100%;display:flex;flex-direction:column;text-align:left;gap:4px;padding:11px 13px;margin-top:8px;border-radius:9px;border:1px solid #314c60;background:#172d3e;color:#d5ecf7}
  .ff-preset strong{font-size:12px}
  .ff-preset span{font-size:10px;color:#96b5c5;line-height:1.55}
  .ff-preset:hover{background:#204154;border-color:#529cb0}
  .ff-file-input{width:1px;height:1px;opacity:0;position:absolute}
  .ff-upload{display:inline-block;cursor:pointer;margin:4px 0 13px}
  .ff-file-input:focus-visible + .ff-upload{outline:2px solid #b5e6ef}
  .ff-paste-label{display:block;color:#b5d0dc;font-size:11px;font-weight:700;margin-bottom:7px}
  .ff-json-input{display:block;resize:vertical;width:100%;min-height:110px;border:1px solid #30465b;border-radius:9px;background:#0d1724;color:#cbe8f6;padding:10px;font-family:ui-monospace,monospace;font-size:11px;line-height:1.6}
  .ff-json-input:focus{outline:1px solid #75d2ea}
  .ff-actions{display:flex;flex-wrap:wrap;gap:7px;margin-top:10px}
  .ff-source{color:#83dceb;font-size:11px;text-decoration:none}
  .ff-source:hover{text-decoration:underline}
  @media(max-width:1200px){.ff-layout{grid-template-columns:minmax(0,1fr)}.ff-side{display:grid;grid-template-columns:repeat(2,minmax(0,1fr))}.ff-side .ff-box:last-child{grid-column:1/-1}}
  @media(max-width:850px){.ff-side{display:flex}.ff-status{flex-direction:column}.ff-row{align-items:flex-start;flex-direction:column}.ff-row-input{flex:0 0 auto;width:100%}}
</style>
