<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import UiIcon from "./UiIcon.svelte";
  import {
    openRobloxDownloadPage, openRobloxInstallFolder,
    robloxInstallStatus, setRobloxInstallPath
  } from "./lib/backend";
  import type { RobloxInstallStatus } from "./lib/backend";

  let status: RobloxInstallStatus | null = null;
  let busy = false;
  let pathText = "";
  let message = "";
  let error = "";

  onMount(() => { void detect(); });

  async function detect() {
    if (busy) return;
    busy = true;
    error = "";
    message = "";
    try {
      status = await robloxInstallStatus();
      if (status.path) pathText = status.path;
    } catch (reason) {
      error = "Could not check Roblox installation: " + String(reason);
    } finally {
      busy = false;
    }
  }

  async function save(path: string | null) {
    if (busy) return;
    busy = true;
    error = "";
    message = "";
    try {
      status = await setRobloxInstallPath(path);
      pathText = status.path ?? "";
      message = path === null
        ? "Custom path cleared. Thirst is using automatic detection."
        : "Validated Roblox installation path saved.";
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  async function browse() {
    if (busy) return;
    error = "";
    try {
      // Select an .app directory on macOS or the actual RobloxPlayerBeta.exe
      // on Windows. The selected path is validated again by native Rust.
      const mac = navigator.userAgent.includes("Mac");
      const chosen = await open({
        title: mac ? "Select Roblox.app" : "Select RobloxPlayerBeta.exe",
        multiple: false,
        directory: mac,
        canCreateDirectories: false,
        ...(mac ? {} : { filters: [{ name: "Roblox Player", extensions: ["exe"] }] })
      });
      if (typeof chosen === "string" && chosen.length) {
        pathText = chosen;
        await save(chosen);
      }
    } catch (reason) {
      error = "Could not open the file picker: " + String(reason);
    }
  }

  async function reveal() {
    error = "";
    try { await openRobloxInstallFolder(); }
    catch (reason) { error = String(reason); }
  }

  async function install() {
    error = "";
    try { await openRobloxDownloadPage(); }
    catch (reason) { error = String(reason); }
  }
</script>

<section class="panel roblox-install-panel" aria-labelledby="roblox-install-title">
  <div class="roblox-install-heading">
    <div>
      <span class="install-kicker">ROBLOX & LAUNCH</span>
      <h3 id="roblox-install-title">Roblox installation</h3>
      <p>Check the installed Roblox Player and its launch handler. Thirst doesn't download or replace Roblox.</p>
    </div>
    <span class="install-badge" class:found={status?.installed} class:missing={status && !status.installed}>
      {#if busy}CHECKING…
      {:else if status?.installed}FOUND
      {:else if status}NOT FOUND
      {:else}NOT CHECKED
      {/if}
    </span>
  </div>

  {#if status}
    <div class="install-facts">
      <div class="install-fact">
        <span>Installation</span>
        <strong>{status.installed ? status.customPath && status.source === "custom" ? "Custom location" : "Auto-detected" : "Not found"}</strong>
      </div>
      <div class="install-fact">
        <span>Roblox version</span>
        <strong>{status.version ?? "Not available"}</strong>
      </div>
      <div class="install-fact">
        <span>roblox:// handler</span>
        <strong class:problem={status.protocol === "missing" || status.protocol === "other-app"}>
          {status.protocol === "registered" ? "Registered" :
           status.protocol === "missing" ? "Missing" :
           status.protocol === "other-app" ? "Different handler" : "Unverified"}
        </strong>
      </div>
      <div class="install-fact">
        <span>Roblox process</span>
        <strong>{status.running === true ? "Running" : status.running === false ? "Not running" : "Unknown"}</strong>
      </div>
    </div>
    <p class="install-detail">{status.detail}</p>
    {#if status.path}
      <div class="install-detected-path"><span>Detected location</span><code title={status.path}>{status.path}</code></div>
    {/if}
  {:else}
    <p class="install-detail">Detection starts when you open Settings. Select Re-detect to check again.</p>
  {/if}

  <div class="install-actions">
    <button class="btn secondary small" disabled={busy} onclick={detect}><UiIcon name="refresh" size={15}/> Re-detect</button>
    <button class="btn secondary small" disabled={busy} onclick={browse}><UiIcon name="library" size={15}/> Browse…</button>
    <button class="btn secondary small" disabled={busy || !status?.installed} onclick={reveal}>Show in folder</button>
    <button class="btn secondary small" onclick={install}><UiIcon name="arrow-up-right" size={14}/> Official download</button>
  </div>

  <div class="install-manual">
    <label for="roblox-path-field">Manual Roblox installation path</label>
    <p>macOS: select <code>Roblox.app</code>. Windows: select <code>RobloxPlayerBeta.exe</code>. Paths are validated before saving.</p>
    <div class="install-path-controls">
      <input id="roblox-path-field" type="text" bind:value={pathText}
        placeholder={navigator.userAgent.includes("Mac") ? "/Applications/Roblox.app" : "C:\\Users\\…\\RobloxPlayerBeta.exe"}
        spellcheck="false" autocomplete="off"/>
      <button class="btn secondary small" disabled={busy || !pathText.trim()} onclick={() => void save(pathText.trim())}>Use path</button>
      {#if status?.customPath}
        <button class="btn secondary small" disabled={busy} onclick={() => void save(null)}>Clear override</button>
      {/if}
    </div>
  </div>

  {#if message}<p class="install-feedback" role="status">{message}</p>{/if}
  {#if error}<p class="install-error" role="alert">{error}</p>{/if}
  <p class="install-footnote">Detection is local and on demand. A registered handler or running process doesn't guarantee that a specific Roblox game joined successfully. Use Game History to confirm a session.</p>
</section>

<style>
  .roblox-install-panel{
    display:flex;flex-direction:column;gap:17px;padding:22px;
    border:1px solid #345268;border-radius:15px;background:#122232;
  }
  .roblox-install-heading{display:flex;align-items:flex-start;justify-content:space-between;gap:16px}
  .install-kicker{font-weight:850;letter-spacing:1.5px;font-size:9px;color:#83bccc}
  h3{font-size:17px;color:#eff9ff;margin:7px 0 6px}
  .roblox-install-heading p{color:#93afbf;font-size:11px;line-height:1.65;max-width:650px;margin:0}
  .install-badge{flex:none;border:1px solid #486377;background:#1d3445;border-radius:7px;padding:7px 10px;
    color:#bddce8;font-size:9px;letter-spacing:.7px;font-weight:800}
  .install-badge.found{border-color:#4f8d82;color:#a4e9dc;background:#173c37}
  .install-badge.missing{border-color:#875a5b;color:#f0b3ba;background:#3b252e}
  .install-facts{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:8px}
  .install-fact{display:flex;flex-direction:column;gap:8px;padding:13px;border:1px solid #2e4a5e;border-radius:10px;background:#152b3d;min-width:0}
  .install-fact span{color:#88a8bb;font-size:10px;line-height:1.3}
  .install-fact strong{color:#def5ff;font-size:12px;line-height:1.5;overflow-wrap:anywhere}
  .install-fact strong.problem{color:#f0b5bd}
  .install-detail{font-size:11px;line-height:1.65;color:#abc5d1;margin:0}
  .install-detected-path{display:flex;flex-direction:column;gap:8px;min-width:0}
  .install-detected-path>span{color:#9fbfcd;font-size:10px;font-weight:750}
  .install-detected-path code{display:block;max-width:100%;padding:10px;border:1px solid #355469;
    border-radius:9px;background:#0d1b2b;color:#bce8f0;font-size:11px;overflow-wrap:anywhere;user-select:text}
  .install-actions{display:flex;gap:8px;flex-wrap:wrap}
  .install-actions .btn{display:flex;align-items:center;justify-content:center;gap:7px;padding:9px 11px;min-height:35px}
  .install-manual{display:flex;flex-direction:column;gap:8px;padding-top:16px;border-top:1px solid #2d4658}
  .install-manual label{font-size:11px;color:#d8eef7;font-weight:800}
  .install-manual p{font-size:10px;line-height:1.55;color:#8aa6b8;margin:0}
  .install-path-controls{display:flex;gap:8px;flex-wrap:wrap}
  .install-path-controls input{flex:1 1 250px;min-width:0;max-width:100%;border:1px solid #3b5c70;
    border-radius:9px;background:#0e2030;color:#ecf9ff;font-size:11px;padding:10px 12px}
  .install-path-controls input:focus-visible{outline:2px solid #96e6ef;outline-offset:2px}
  .install-path-controls .btn{min-height:36px;padding:9px 12px}
  .install-feedback,.install-error{font-size:11px;line-height:1.5;margin:0}
  .install-feedback{color:#9edcc9}.install-error{color:#f2b2bc}
  .install-footnote{font-size:10px;color:#789bad;line-height:1.6;margin:0}
  @media(max-width:1000px){.install-facts{grid-template-columns:repeat(2,minmax(0,1fr))}}
  @media(max-width:760px){.roblox-install-panel{padding:16px}.install-facts{grid-template-columns:repeat(2,minmax(0,1fr))}}
</style>
