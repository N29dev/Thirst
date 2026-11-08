<script lang="ts">
  import { onMount } from "svelte";
  import { readFastFlags, writeFastFlags } from "./lib/backend";
  import {
    PLAYER_FLAGS, FFLAG_PRESETS, cleanFlagValue, exportFFlagsJson,
    exportNamedFFlagsJson, flagDefinition, importFFlagsJson
  } from "./lib/fastflags";
  import type { FFlagStatus, FFlagPreset } from "./lib/fastflags";

  type ImportMode = "replace" | "merge";
  type ExportMode = "preset" | "roblox";
  let status: FFlagStatus | null = null;
  let draft: Record<string, string> = {};
  let saved: Record<string, string> = {};
  // Imported preset rows marked enabled:false are not written to Roblox,
  // but remain editable and round-trip when exporting a named preset.
  let inactiveFlags: Record<string, string> = {};
  let presetName = "My Fast Flags";
  let importMode: ImportMode = "replace";
  let exportMode: ExportMode = "preset";
  let isBusy = false;
  let dragging = false;
  let error = "";
  let info = "";
  let search = "";
  let jsonInput = "";
  let newFlagName = "";
  const presetNames: FFlagPreset[] = ["low", "balanced", "quality"];

  $: activeKeys = Object.keys(draft).sort();
  $: inactiveKeys = Object.keys(inactiveFlags).sort();
  $: activeDefinitions = PLAYER_FLAGS.filter(flag =>
    Object.prototype.hasOwnProperty.call(draft, flag.name) &&
    (flag.label.toLowerCase().includes(search.toLowerCase()) ||
      flag.name.toLowerCase().includes(search.toLowerCase())));
  $: inactiveDefinitions = PLAYER_FLAGS.filter(flag =>
    Object.prototype.hasOwnProperty.call(inactiveFlags, flag.name) &&
    (flag.label.toLowerCase().includes(search.toLowerCase()) ||
      flag.name.toLowerCase().includes(search.toLowerCase())));
  $: unusedDefinitions = PLAYER_FLAGS.filter(flag =>
    !Object.prototype.hasOwnProperty.call(draft, flag.name) &&
    !Object.prototype.hasOwnProperty.call(inactiveFlags, flag.name));
  $: dirty = JSON.stringify(Object.entries(draft).sort()) !==
    JSON.stringify(Object.entries(saved).sort());

  onMount(() => { void reload(); });

  async function reload() {
    if (isBusy) return;
    isBusy = true;
    error = "";
    try {
      const result = await readFastFlags();
      status = result;
      draft = { ...result.flags };
      saved = { ...result.flags };
      inactiveFlags = {};
      info = result.fileExists
        ? "Current Roblox settings loaded. Changes are not applied until you save."
        : "No settings file yet. Import a preset or choose a graphics profile.";
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
      const canonical: Record<string, string> = {};
      for (const [name, value] of Object.entries(draft)) {
        canonical[name] = cleanFlagValue(name, value);
      }
      const result = await writeFastFlags(canonical);
      status = result;
      draft = { ...result.flags };
      saved = { ...result.flags };
      info = "Saved supported flags to Roblox Player. Restart Roblox to apply changes.";
    } catch (reason) {
      error = String(reason);
    } finally {
      isBusy = false;
    }
  }

  function applyPreset(name: FFlagPreset) {
    const preset = FFLAG_PRESETS[name];
    const disabled = { ...inactiveFlags };
    for (const key of Object.keys(preset.flags)) delete disabled[key];
    inactiveFlags = disabled;
    draft = { ...draft, ...preset.flags };
    info = preset.label + " profile loaded into the editor. Review and save to apply.";
    error = "";
  }

  function addFlag() {
    const name = unusedDefinitions.some(flag => flag.name === newFlagName)
      ? newFlagName : unusedDefinitions[0]?.name;
    if (!name) return;
    const def = flagDefinition(name);
    if (!def) return;
    draft = { ...draft, [name]: def.type === "boolean" ? "True" : "0" };
    newFlagName = "";
    search = "";
    info = "Added " + name + ". Save to apply.";
  }

  function removeFlag(name: string) {
    const enabled = { ...draft };
    const disabled = { ...inactiveFlags };
    delete enabled[name];
    delete disabled[name];
    draft = enabled;
    inactiveFlags = disabled;
  }

  function disableFlag(name: string) {
    if (!Object.prototype.hasOwnProperty.call(draft, name)) return;
    inactiveFlags = { ...inactiveFlags, [name]: draft[name] };
    const next = { ...draft };
    delete next[name];
    draft = next;
  }

  function enableFlag(name: string) {
    if (!Object.prototype.hasOwnProperty.call(inactiveFlags, name)) return;
    const value = inactiveFlags[name];
    const next = { ...inactiveFlags };
    delete next[name];
    inactiveFlags = next;
    setFlag(name, value);
  }

  function setFlag(name: string, value: string) {
    const next = { ...draft, [name]: value };
    if (value === "True" && [
      "FFlagDebugGraphicsPreferD3D11",
      "FFlagDebugGraphicsPreferVulkan",
      "FFlagDebugGraphicsPreferOpenGL"
    ].includes(name)) {
      for (const key of [
        "FFlagDebugGraphicsPreferD3D11",
        "FFlagDebugGraphicsPreferVulkan",
        "FFlagDebugGraphicsPreferOpenGL"
      ]) {
        if (key !== name && Object.prototype.hasOwnProperty.call(next, key)) next[key] = "False";
      }
    }
    draft = next;
  }

  function importJson(raw: string) {
    try {
      const result = importFFlagsJson(raw);
      const enabled = importMode === "replace" ? {} as Record<string, string> : { ...draft };
      const disabled = importMode === "replace" ? {} as Record<string, string> : { ...inactiveFlags };
      for (const [name, value] of Object.entries(result.flags)) {
        enabled[name] = value;
        delete disabled[name];
      }
      for (const [name, value] of Object.entries(result.disabled)) {
        disabled[name] = value;
        delete enabled[name];
      }
      draft = enabled;
      inactiveFlags = disabled;
      if (result.name) presetName = result.name;
      const skipped = result.ignored.length + result.invalid.length;
      info = "Imported " + Object.keys(result.flags).length + " enabled and " +
        Object.keys(result.disabled).length + " disabled flags from " +
        (result.name ? '"' + result.name + '"' : "Roblox settings") +
        (skipped ? "; skipped " + skipped + " unsupported or invalid entries." : ".") +
        " Review the draft and click Save to Roblox when ready.";
      error = "";
    } catch (reason) {
      error = String(reason);
    }
  }

  async function importFileContent(file: File) {
    try {
      if (file.size > 64_000) throw new Error("The JSON file exceeds 64 KB.");
      importJson(await file.text());
    } catch (reason) {
      error = String(reason);
    }
  }

  async function importFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (file) await importFileContent(file);
    input.value = "";
  }

  function dragOver(event: DragEvent) {
    event.preventDefault();
    dragging = true;
  }

  function dragLeave(event: DragEvent) {
    event.preventDefault();
    dragging = false;
  }

  function dropFile(event: DragEvent) {
    event.preventDefault();
    dragging = false;
    const file = event.dataTransfer?.files?.[0];
    if (file) void importFileContent(file);
  }

  function exportContent(): string {
    return exportMode === "preset"
      ? exportNamedFFlagsJson(draft, inactiveFlags, presetName)
      : exportFFlagsJson(draft);
  }

  async function copyJson() {
    try {
      await navigator.clipboard.writeText(exportContent());
      info = "Copied " + (exportMode === "preset" ? "named preset" : "Roblox settings") + " JSON.";
      error = "";
    } catch (reason) {
      error = "Could not copy JSON: " + String(reason);
    }
  }

  function downloadJson() {
    try {
      const content = exportContent();
      const blob = new Blob([content], { type: "application/json;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      const safeName = presetName.toLowerCase().trim()
        .replace(/[^a-z0-9_-]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 55) || "thirst-preset";
      link.download = exportMode === "preset" ? safeName + ".json" : "ClientAppSettings.json";
      document.body.appendChild(link);
      link.click();
      link.remove();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      info = "Exported " + link.download + ". Your Roblox session was not included.";
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
      error = "Clipboard access is unavailable.";
    }
  }
</script>

<div class="ff-page">
  <div class="page-title ff-title">
    <div>
      <span class="eyebrow">CONFIGURATION / ROBLOX PLAYER</span>
      <h1>Fast Flags<span class="ff-title-dot">.</span></h1>
      <p>A cleaner way to manage, import, and share your Roblox graphics presets.</p>
    </div>
    <div class="ff-top-actions">
      <label for="ff-file" class="btn secondary ff-top-import">↑ Import preset</label>
      <button class="btn primary" onclick={downloadJson}>↓ Export {exportMode === "preset" ? "preset" : "JSON"}</button>
    </div>
  </div>

  {#if error}<div class="ff-message ff-error" role="alert">{error}</div>{/if}
  {#if info}<div class="ff-message" role="status">{info}</div>{/if}

  <div class="ff-overview">
    <div class="ff-overview-card"><span>ENABLED FLAGS</span><strong>{activeKeys.length}<small> / {PLAYER_FLAGS.length}</small></strong><p>Ready to write to Roblox</p></div>
    <div class="ff-overview-card"><span>DISABLED IN PRESET</span><strong>{inactiveKeys.length.toString().padStart(2, "0")}</strong><p>Kept for preset exports</p></div>
    <div class="ff-overview-card"><span>PLAYER CONFIG</span><strong class="ff-config-state">{status?.fileExists ? "Detected" : "Not created"}</strong><p>{status?.platform ?? "Checking your device"}</p></div>
  </div>

  <div class="ff-workspace">
    <div class="ff-main">
      <section class="panel ff-editor">
        <div class="ff-section-head">
          <div><span class="ff-kicker">YOUR CONFIGURATION</span><h2>Flags editor</h2><p>Search, edit, enable, or disable supported Roblox Player flags.</p></div>
          <span class="ff-dirty" class:changed={dirty}>{dirty ? "● Unsaved changes" : "✓ Saved to disk"}</span>
        </div>
        <div class="ff-controls">
          <label for="ff-search">Search your flags
            <input id="ff-search" class="input" placeholder="Search name or description…" bind:value={search}/>
          </label>
          <div class="ff-add">
            <label for="ff-new">Add a flag
              <select id="ff-new" class="input" bind:value={newFlagName} disabled={!unusedDefinitions.length}>
                <option value="">Choose supported flag</option>
                {#each unusedDefinitions as option (option.name)}
                  <option value={option.name}>{option.label}</option>
                {/each}
              </select>
            </label>
            <button class="btn secondary" disabled={!unusedDefinitions.length || isBusy} onclick={addFlag}>+ Add</button>
          </div>
        </div>

        <div class="ff-list-caption"><span>ENABLED · {activeKeys.length}</span><span>CHANGES APPLY AFTER SAVE</span></div>
        <div class="ff-table">
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
                    onchange={e => setFlag(flag.name, e.currentTarget.value)}>
                    <option value="True">True</option>
                    <option value="False">False</option>
                  </select>
                {:else}
                  <input class="input" type="number" step="1" aria-label={flag.name}
                    value={draft[flag.name]} oninput={e => setFlag(flag.name, e.currentTarget.value)}/>
                {/if}
                <button class="ff-icon-btn" title={"Disable " + flag.name + " in this preset"}
                  aria-label={"Disable " + flag.name} onclick={() => disableFlag(flag.name)}>◌</button>
                <button class="ff-icon-btn danger" title={"Remove " + flag.name}
                  aria-label={"Remove " + flag.name} onclick={() => removeFlag(flag.name)}>×</button>
              </div>
            </div>
          {:else}
            <div class="ff-empty">{activeKeys.length ? "No enabled flags match your search." :
              "Start with an imported preset, a graphics profile, or a supported flag."}</div>
          {/each}
        </div>

        {#if inactiveDefinitions.length}
          <div class="ff-list-caption ff-disabled-caption"><span>DISABLED IN PRESET · {inactiveKeys.length}</span><span>NOT APPLIED TO ROBLOX</span></div>
          <div class="ff-disabled-list">
            {#each inactiveDefinitions as flag (flag.name)}
              <div class="ff-disabled-row">
                <div><strong>{flag.label}</strong><code>{flag.name}</code></div>
                <span class="ff-disabled-value">{inactiveFlags[flag.name]}</span>
                <button class="btn secondary small" onclick={() => enableFlag(flag.name)}>Enable</button>
                <button class="ff-icon-btn danger" aria-label={"Remove " + flag.name} onclick={() => removeFlag(flag.name)}>×</button>
              </div>
            {/each}
          </div>
        {/if}
        <div class="ff-savebar">
          <div><strong>{dirty ? "Changes ready to apply" : "Player settings are up to date"}</strong><small>Only enabled and allowlisted flags are written to Roblox.</small></div>
          <div class="ff-save-actions">
            <button class="btn secondary" onclick={reload} disabled={isBusy || !status || !dirty}>Discard</button>
            <button class="btn primary" onclick={save} disabled={isBusy || !status || !dirty}>{isBusy ? "Saving…" : "Save to Roblox"}</button>
          </div>
        </div>
      </section>
    </div>

    <aside class="ff-sidebar">
      <section class="panel ff-box ff-transfer">
        <span class="ff-kicker">PRESET MANAGEMENT</span>
        <h2>Import & export</h2>
        <p>Supports both raw ClientAppSettings.json and named presets such as <strong>better-gk.json</strong>.</p>
        <input class="ff-file-input" id="ff-file" type="file" accept=".json,application/json" onchange={importFile}/>
        <label for="ff-file" class="ff-dropzone" class:dragging={dragging}
          ondragover={dragOver} ondragleave={dragLeave} ondrop={dropFile}>
          <span class="ff-drop-symbol">↥</span>
          <strong>Drop JSON here or browse</strong>
          <small>Preset JSON / Roblox ClientAppSettings</small>
        </label>
        <label class="ff-field-label" for="ff-import-mode">When importing</label>
        <select class="input" id="ff-import-mode" bind:value={importMode}>
          <option value="replace">Replace current draft</option>
          <option value="merge">Merge with current draft</option>
        </select>
        <details class="ff-paste-details">
          <summary>Paste JSON instead</summary>
          <textarea id="ff-json" aria-label="Paste Fast Flags JSON" rows="5" class="ff-json-input" bind:value={jsonInput}
            placeholder="Paste named preset or ClientAppSettings JSON"></textarea>
          <button class="btn secondary small" disabled={!jsonInput.trim()} onclick={() => importJson(jsonInput)}>Import pasted JSON</button>
        </details>
        <div class="ff-transfer-divider"></div>
        <label class="ff-field-label" for="ff-preset-name">Preset name</label>
        <input id="ff-preset-name" class="input" maxlength="70" bind:value={presetName} placeholder="Name your preset"/>
        <label class="ff-field-label" for="ff-export-type">Export format</label>
        <select id="ff-export-type" class="input" bind:value={exportMode}>
          <option value="preset">Named preset (keeps disabled flags)</option>
          <option value="roblox">Roblox settings (enabled only)</option>
        </select>
        <div class="ff-export-actions">
          <button class="btn primary" onclick={downloadJson}>↓ Download JSON</button>
          <button class="btn secondary" onclick={copyJson}>Copy</button>
        </div>
        <p class="ff-explain">Named presets are shareable. Exports contain no login cookies. Disabled entries are excluded from the Roblox settings format.</p>
      </section>

      <section class="panel ff-box">
        <span class="ff-kicker">ONE-CLICK SETTINGS</span>
        <h2>Graphics profiles</h2>
        <p>Small, reversible starting points. These don't erase your other flags.</p>
        <div class="ff-presets">
          {#each presetNames as name (name)}
            <button class="ff-preset" onclick={() => applyPreset(name)}>
              <span class="ff-preset-glyph">{name === "low" ? "◔" : name === "balanced" ? "◑" : "●"}</span>
              <span><strong>{FFLAG_PRESETS[name].label}</strong><small>{FFLAG_PRESETS[name].description}</small></span>
              <span class="ff-preset-arrow">↗</span>
            </button>
          {/each}
        </div>
      </section>

      <section class="panel ff-box ff-install">
        <span class="ff-kicker">LOCAL ROBLOX PLAYER</span>
        <h2>Installation</h2>
        {#if status}
          <div class="ff-status-line"><span>Configuration</span><strong>{status.fileExists ? "Found" : "Not created"}</strong></div>
          <div class="ff-status-line"><span>Platform</span><strong>{status.platform}</strong></div>
          {#if !status.playerDetected}<p class="ff-install-warning">Roblox installation wasn't detected. You can still edit a draft.</p>{/if}
          <button class="ff-path" title="Copy settings location" onclick={copyPath}>{status.path} <span>⧉</span></button>
          <button class="btn secondary small" onclick={reload} disabled={isBusy}>↻ Reload settings</button>
          {#if status.ignoredKeys.length}
            <p>{status.ignoredKeys.length} unsupported entries are present on disk; Thirst leaves them untouched.</p>
          {/if}
        {:else}
          <p>Loading your Player configuration…</p>
        {/if}
        <p class="ff-explain">Thirst makes a one-time backup before overwriting existing Player settings. Restart Roblox after saving.</p>
        <a class="ff-source" href="https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569/" target="_blank" rel="noopener noreferrer">Roblox Player allowlist ↗</a>
      </section>
    </aside>
  </div>
</div>

<style>
  .ff-page{min-width:0}
  .ff-title{align-items:center;gap:16px}
  .ff-title h1{font-size:clamp(30px,3vw,44px);letter-spacing:-1.8px;margin:9px 0 4px}
  .ff-title-dot{color:#77dcea}
  .ff-title p{max-width:660px;font-size:13px;line-height:1.7}
  .ff-top-actions{display:flex;gap:9px;align-items:center;flex-wrap:wrap}
  .ff-top-import{cursor:pointer}
  .ff-message{border:1px solid #34596a;background:#163040;color:#c1f3f9;font-size:12px;line-height:1.65;padding:13px 17px;border-radius:12px;margin-bottom:15px;overflow-wrap:anywhere}
  .ff-message.ff-error{border-color:#7c3d4a;background:#2d1d2a;color:#ffbac7}
  .ff-overview{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:13px;margin-bottom:17px}
  .ff-overview-card{padding:20px 21px;background:linear-gradient(155deg,#172638,#111b29 76%);border:1px solid #314251;border-radius:16px;min-width:0}
  .ff-overview-card>span{font-size:10px;letter-spacing:1.4px;font-weight:800;color:#8cacc3}
  .ff-overview-card strong{display:block;font-size:30px;letter-spacing:-1px;margin-top:12px;color:#e9fbff}
  .ff-overview-card strong small{font-size:15px;color:#6e95a8}
  .ff-overview-card .ff-config-state{font-size:21px;margin-top:15px;letter-spacing:-.45px}
  .ff-overview-card p{font-size:11px;color:#879dae;margin:6px 0 0}
  .ff-workspace{display:grid;grid-template-columns:minmax(0,1.65fr) minmax(270px,.92fr);gap:16px;align-items:start}
  .ff-main,.ff-sidebar{min-width:0}
  .ff-sidebar{display:flex;flex-direction:column;gap:16px}
  .ff-editor{overflow:hidden}
  .ff-section-head{padding:25px 25px 19px;display:flex;justify-content:space-between;align-items:flex-start;gap:14px}
  .ff-kicker{display:block;color:#78cddf;letter-spacing:1.5px;font-size:9px;font-weight:850;margin-bottom:11px}
  .ff-section-head h2,.ff-box h2{font-size:19px;line-height:1.2;letter-spacing:-.55px;margin:0 0 7px}
  .ff-section-head p,.ff-box p{font-size:11px;line-height:1.75;color:#90a4b6;margin:0}
  .ff-dirty{color:#85bea9;font-size:10px;white-space:nowrap;background:#15352d;border:1px solid #255348;padding:7px 10px;border-radius:30px}
  .ff-dirty.changed{color:#f4cc8f;background:#3a2d1f;border-color:#70532d}
  .ff-controls{padding:0 25px 20px;display:flex;gap:12px;flex-wrap:wrap;align-items:end}
  .ff-controls>label,.ff-add label{display:flex;flex-direction:column;gap:9px;color:#adbfce;font-size:11px;font-weight:650;flex:1;min-width:180px}
  .ff-add{display:flex;align-items:end;gap:8px;flex:1.15;min-width:245px}
  .ff-add .btn{min-height:42px}
  .ff-list-caption{padding:13px 25px;display:flex;justify-content:space-between;align-items:center;gap:10px;background:#102030;border-top:1px solid #293c4a;border-bottom:1px solid #293c4a;color:#83c4d1;font-size:9px;letter-spacing:1.1px;font-weight:800}
  .ff-list-caption span:last-child{color:#617f95}
  .ff-disabled-caption{margin-top:5px;color:#c9b797}
  .ff-table{max-height:610px;overflow-y:auto}
  .ff-row{display:flex;gap:12px;justify-content:space-between;align-items:center;padding:18px 25px;border-bottom:1px solid #243543;transition:background .15s}
  .ff-row:hover{background:#162b39}
  .ff-row-info{display:flex;flex-direction:column;gap:5px;min-width:0;overflow-wrap:anywhere}
  .ff-row-info strong{font-size:12px;color:#e3f1fa}
  .ff-row-info code,.ff-disabled-row code{font-size:10px;color:#71b8c9;word-break:break-word}
  .ff-row-info small{font-size:10px;line-height:1.6;color:#839cb0}
  .ff-row-input{display:flex;align-items:center;gap:7px;flex:0 0 205px}
  .ff-row-input .input{height:36px;min-width:0}
  .ff-icon-btn{border:1px solid #335164;background:#183044;color:#bde5ef;display:grid;place-items:center;width:33px;height:35px;flex:none;border-radius:9px;font-size:16px}
  .ff-icon-btn:hover{background:#26475d;border-color:#639cb6}
  .ff-icon-btn.danger{color:#f0a9b2;border-color:#513b48;background:#2c202c}
  .ff-icon-btn.danger:hover{background:#4a2836}
  .ff-disabled-list{padding:2px 0}
  .ff-disabled-row{display:flex;align-items:center;gap:9px;padding:11px 25px;border-bottom:1px solid #263747}
  .ff-disabled-row>div{display:flex;flex-direction:column;gap:5px;flex:1;min-width:0}
  .ff-disabled-row strong{font-size:11px}
  .ff-disabled-value{font-family:ui-monospace,monospace;font-size:11px;color:#a2bdd0;max-width:70px;overflow:hidden;text-overflow:ellipsis}
  .ff-empty{padding:53px 24px;text-align:center;font-size:12px;line-height:1.8;color:#879aac}
  .ff-savebar{position:sticky;bottom:0;display:flex;justify-content:space-between;align-items:center;gap:14px;flex-wrap:wrap;padding:19px 25px;background:#111e2ccc;border-top:1px solid #344859;backdrop-filter:blur(16px)}
  .ff-savebar strong,.ff-savebar small{display:block}
  .ff-savebar strong{font-size:12px}
  .ff-savebar small{font-size:10px;line-height:1.6;margin-top:4px;color:#819bad}
  .ff-save-actions{display:flex;gap:9px}
  .ff-box{padding:22px}
  .ff-box h2{margin-bottom:9px}
  .ff-transfer>p strong{color:#d9f1fc}
  .ff-file-input{position:absolute;width:1px;height:1px;opacity:0}
  .ff-file-input:focus-visible ~ .ff-workspace .ff-dropzone{outline:2px solid #89dfe9}
  .ff-dropzone{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:7px;margin:17px 0 18px;padding:24px 12px;border:1px dashed #47758c;background:linear-gradient(135deg,#162e3d,#132331);border-radius:14px;cursor:pointer;text-align:center;transition:background .15s,border-color .15s}
  .ff-dropzone:hover,.ff-dropzone.dragging{background:#1e3e50;border-color:#83d3e5}
  .ff-drop-symbol{display:grid;place-items:center;width:40px;height:40px;border:1px solid #47768d;border-radius:13px;background:#1d455a;font-size:26px;color:#9eebf6;margin-bottom:4px}
  .ff-dropzone strong{font-size:12px;color:#e4f4fc}
  .ff-dropzone small{font-size:10px;color:#8ba9bb;line-height:1.5}
  .ff-field-label{display:block;font-size:10px;color:#abc6d5;margin:14px 0 8px;font-weight:750}
  .ff-box .input{height:40px}
  .ff-paste-details{margin-top:15px}
  .ff-paste-details summary{cursor:pointer;font-size:11px;color:#86d9ea;font-weight:700}
  .ff-paste-details summary:hover{text-decoration:underline}
  .ff-paste-details .btn{margin-top:9px}
  .ff-json-input{width:100%;min-height:96px;resize:vertical;background:#0b1926;border:1px solid #324d61;color:#cbe6f4;border-radius:9px;padding:12px;margin-top:10px;font-size:11px;line-height:1.7;font-family:ui-monospace,monospace}
  .ff-transfer-divider{margin:22px 0 16px;height:1px;background:#304252}
  .ff-export-actions{display:flex;gap:9px;margin-top:17px;flex-wrap:wrap}
  .ff-export-actions .btn:first-child{flex:1}
  .ff-explain{font-size:10px!important;color:#7897aa!important;line-height:1.7!important;margin-top:14px!important}
  .ff-presets{display:flex;flex-direction:column;gap:8px;margin-top:17px}
  .ff-preset{width:100%;display:flex;align-items:center;gap:11px;background:#142b3b;border:1px solid #2d4b5e;border-radius:12px;padding:12px;text-align:left;color:#d9edf7}
  .ff-preset:hover{background:#204052;border-color:#6799b1}
  .ff-preset-glyph{display:grid;place-items:center;width:37px;height:37px;border-radius:10px;flex:none;background:#205064;color:#9de8f4;font-size:22px}
  .ff-preset>span:nth-child(2){min-width:0;flex:1}
  .ff-preset strong,.ff-preset small{display:block}
  .ff-preset strong{font-size:12px}
  .ff-preset small{font-size:10px;line-height:1.6;color:#8aafc0;margin-top:3px}
  .ff-preset-arrow{color:#6aadc4}
  .ff-status-line{display:flex;justify-content:space-between;font-size:11px;color:#8dabbc;margin:12px 0;gap:10px}
  .ff-status-line strong{font-weight:700;color:#d0eef5}
  .ff-path{display:block;text-align:left;margin:18px 0 12px;background:#102434;border:1px solid #2c4a5d;border-radius:9px;padding:10px;color:#92d4df;font-size:10px;line-height:1.7;word-break:break-all;cursor:pointer;width:100%}
  .ff-path:hover{border-color:#67b3c8}
  .ff-install-warning{color:#e8b694!important}
  .ff-source{display:inline-block;color:#84d7e5;font-size:11px;text-decoration:none;margin-top:12px}
  .ff-source:hover{text-decoration:underline}
  @media(max-width:1150px){.ff-workspace{grid-template-columns:minmax(0,1fr)}.ff-sidebar{display:grid;grid-template-columns:repeat(2,minmax(0,1fr))}.ff-transfer{grid-row:span 2}}
  @media(max-width:770px){.ff-overview{grid-template-columns:1fr 1fr}.ff-workspace,.ff-sidebar{display:flex;flex-direction:column}.ff-sidebar>.panel{width:100%}.ff-title{align-items:flex-start;flex-direction:column}.ff-title .ff-top-actions{width:100%}.ff-row{align-items:flex-start;flex-direction:column}.ff-row-input{flex:0 0 auto;width:100%}.ff-overview-card{padding:16px}.ff-section-head{flex-wrap:wrap}.ff-disabled-row{padding:12px}.ff-savebar{padding:15px}.ff-controls{padding:0 15px 18px}.ff-list-caption{padding:12px 15px}.ff-row{padding:15px}.ff-section-head{padding:20px 15px}}
</style>
