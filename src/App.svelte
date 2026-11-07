<script lang="ts">
  import { onMount } from "svelte";
  import { fetchPublicServers, launchPlace, platformName } from "./lib/backend";
  import { parsePlaceId, readLibrary, readLiteMode, writeLibrary, writeLiteMode } from "./lib/library";
  import type { PublicServer, SavedGame, SavedPlace } from "./lib/types";

  type Tab = "home" | "library" | "servers" | "settings";
  let tab: Tab = "home";
  let games: SavedGame[] = [];
  let selectedId: number | null = null;
  let activePlaceId: number | null = null;
  let newName = "";
  let newPlaceId = "";
  let newSubplaceName = "";
  let newSubplaceId = "";
  let filter = "";
  let liteMode = false;
  let platform = "Desktop";
  let toast = "";
  let toastKind: "good" | "bad" = "good";
  let loading = false;
  let servers: PublicServer[] = [];
  let cursor: string | null = null;
  let nextCursor: string | null = null;
  let fetchedAt: Date | null = null;

  $: selected = games.find(game => game.placeId === selectedId) || null;
  $: filteredGames = games.filter(game => game.name.toLowerCase().includes(filter.toLowerCase()) ||
    String(game.placeId).includes(filter));
  $: availablePlaces = selected
    ? [{ placeId: selected.placeId, name: "Main place" }, ...selected.subplaces]
    : [];

  onMount(() => {
    games = readLibrary();
    selectedId = games[0]?.placeId ?? null;
    activePlaceId = selectedId;
    liteMode = readLiteMode();
    platformName().then(value => platform = value).catch(() => platform = "Desktop");
  });

  function notice(message: string, kind: "good" | "bad" = "good") {
    toast = message;
    toastKind = kind;
  }

  function persist() {
    try { writeLibrary(games); } catch { notice("Couldn't save to local storage.", "bad"); }
  }

  function addGame() {
    const id = parsePlaceId(newPlaceId);
    const name = newName.trim().slice(0, 100);
    if (!id || !name) return notice("Enter a game name and valid Roblox place ID or URL.", "bad");
    if (games.some(game => game.placeId === id)) return notice("This game is already in your library.", "bad");
    if (games.length >= 200) return notice("Library limit reached.", "bad");
    games = [...games, { placeId: id, name, subplaces: [], addedAt: Date.now() }];
    persist();
    selectGame(id);
    newName = "";
    newPlaceId = "";
    notice("Saved to your library.");
  }

  function selectGame(id: number) {
    selectedId = id;
    activePlaceId = id;
    servers = [];
    cursor = null;
    nextCursor = null;
    fetchedAt = null;
  }

  function removeGame() {
    if (!selected) return;
    games = games.filter(game => game.placeId !== selected?.placeId);
    persist();
    selectGame(games[0]?.placeId ?? 0);
    if (!games.length) { selectedId = null; activePlaceId = null; }
    notice("Removed from library.");
  }

  function addSubplace() {
    if (!selected) return;
    const id = parsePlaceId(newSubplaceId);
    const name = newSubplaceName.trim().slice(0, 100);
    if (!id || !name) return notice("Enter a subplace name and valid place ID.", "bad");
    if (id === selected.placeId || selected.subplaces.some(place => place.placeId === id))
      return notice("This place is already saved here.", "bad");
    if (selected.subplaces.length >= 100) return notice("Subplace limit reached.", "bad");
    games = games.map(game => game.placeId === selected?.placeId
      ? { ...game, subplaces: [...game.subplaces, { placeId: id, name }] } : game);
    persist();
    newSubplaceName = "";
    newSubplaceId = "";
    notice("Subplace saved.");
  }

  function removeSubplace(place: SavedPlace) {
    if (!selected) return;
    games = games.map(game => game.placeId === selected?.placeId
      ? { ...game, subplaces: game.subplaces.filter(p => p.placeId !== place.placeId) } : game);
    persist();
    if (activePlaceId === place.placeId) changeActivePlace(selected.placeId);
  }

  function changeActivePlace(id: number) {
    activePlaceId = id;
    servers = [];
    cursor = null;
    nextCursor = null;
    fetchedAt = null;
  }

  async function play(placeId: number, jobId: string | null = null) {
    try {
      await launchPlace(placeId, jobId);
      notice(jobId ? "Opening selected server in Roblox…" : "Opening Roblox…");
    } catch (error) {
      notice(String(error), "bad");
    }
  }

  async function loadServers(loadMore = false) {
    if (!activePlaceId || loading) return;
    loading = true;
    try {
      const result = await fetchPublicServers(activePlaceId, loadMore ? nextCursor : null);
      servers = loadMore ? [...servers, ...result.data.filter(s => !servers.some(old => old.id === s.id))] : result.data;
      cursor = loadMore ? nextCursor : null;
      nextCursor = result.nextPageCursor;
      fetchedAt = new Date();
      if (!servers.length) notice("No public servers returned for this place.");
    } catch (error) {
      notice("Could not fetch servers: " + String(error), "bad");
    } finally {
      loading = false;
    }
  }

  function setLiteMode(enabled: boolean) {
    liteMode = enabled;
    try { writeLiteMode(enabled); } catch { notice("Couldn't save preference.", "bad"); }
  }

  function goToServers(id?: number) {
    if (id) changeActivePlace(id);
    tab = "servers";
  }
</script>

<svelte:head>
  <title>Thirst — Stay thirsty for more</title>
  <meta name="description" content="Thirst: a lightweight Roblox launcher for desktop." />
</svelte:head>

<div class:lite={liteMode} class="app">
  <aside class="sidebar">
    <button class="brand" onclick={() => tab = "home"} aria-label="Thirst home">
      <span class="logo">◕</span>
      <span class="brand-text"><strong>thirst<span class="brand-dot">.</span></strong><small>ROBLOX LAUNCHER</small></span>
    </button>
    <span class="section-label">WORKSPACE</span>
    <nav aria-label="Main navigation">
      <button class:active={tab === "home"} onclick={() => tab = "home"}><span class="nav-icon">⌂</span> Overview</button>
      <button class:active={tab === "library"} onclick={() => tab = "library"}><span class="nav-icon">▦</span> My library <span class="nav-count">{games.length}</span></button>
      <button class:active={tab === "servers"} onclick={() => tab = "servers"}><span class="nav-icon">◎</span> Server browser</button>
    </nav>
    <span class="section-label section-spaced">PREFERENCES</span>
    <nav aria-label="Preferences">
      <button class:active={tab === "settings"} onclick={() => tab = "settings"}><span class="nav-icon">⚙</span> Settings</button>
    </nav>
    <div class="sidebar-bottom">
      <div class="sidebar-note"><span class="note-indicator"></span><span>Local-first · No account needed</span></div>
      <div class="version">THIRST ALPHA <span>0.1.0</span></div>
    </div>
  </aside>

  <main>
    <header class="topbar">
      <div class="breadcrumb">THIRST <span>/</span> {tab === "home" ? "OVERVIEW" : tab === "library" ? "LIBRARY" : tab === "servers" ? "SERVERS" : "SETTINGS"}</div>
      <div class="top-actions"><span class="platform-tag">{platform}</span><span class="alpha-tag">EARLY ACCESS</span></div>
    </header>
    <div class="content">
      {#if tab === "home"}
        <div class="hero">
          <div class="hero-glow"></div>
          <div class="hero-content">
            <span class="eyebrow"><span class="tiny-dot"></span> YOUR LAUNCHER. YOUR RULES.</span>
            <h1>Stay thirsty<br /><em>for more.</em></h1>
            <p>One clean place for your favorite Roblox experiences, their subplaces, and public servers.</p>
            <div class="hero-actions">
              <button class="btn primary" onclick={() => selected ? play(selected.placeId) : tab = "library"}>{selected ? "▶ Launch " + selected.name : "+ Add your first game"}</button>
              <button class="btn secondary" onclick={() => tab = "library"}>Open library →</button>
            </div>
          </div>
          <div class="hero-art" aria-hidden="true"><div class="orb orb-outer"><div class="orb orb-inner">T</div></div></div>
        </div>
        <div class="stats">
          <div class="stat"><small>SAVED EXPERIENCES</small><strong>{games.length.toString().padStart(2, "0")}</strong><span>In your personal library</span></div>
          <div class="stat"><small>BOOKMARKED SUBPLACES</small><strong>{games.reduce((n, g) => n + g.subplaces.length, 0).toString().padStart(2, "0")}</strong><span>Quick launch shortcuts</span></div>
          <div class="stat"><small>BACKGROUND REFRESH</small><strong>OFF</strong><span>Servers load only on request</span></div>
        </div>
        <div class="section-heading"><div><span class="eyebrow">QUICK ACCESS</span><h2>Your workspace</h2></div></div>
        <div class="feature-grid">
          <button class="feature" onclick={() => tab = "library"}><div class="feature-symbol">▦</div><h3>Game library</h3><p>Save experiences and individual subplaces. No account required.</p><span class="feature-link">Explore library ↗</span></button>
          <button class="feature" onclick={() => tab = "servers"}><div class="feature-symbol">◎</div><h3>Public servers</h3><p>Browse server population, FPS and Roblox-reported latency.</p><span class="feature-link">Browse servers ↗</span></button>
          <button class="feature" onclick={() => tab = "settings"}><div class="feature-symbol">⚙</div><h3>Personalize</h3><p>Switch to Lite Mode and keep resource usage predictable.</p><span class="feature-link">Your settings ↗</span></button>
        </div>
      {:else if tab === "library"}
        <div class="page-title"><div><span class="eyebrow">YOUR COLLECTION</span><h1>Game library</h1><p>Bookmark experiences and their individual Roblox places.</p></div><span class="count-badge">{games.length} SAVED</span></div>
        <div class="library-layout">
          <div class="panel">
            <div class="panel-header"><h3>Saved experiences</h3></div>
            <input class="input search" aria-label="Filter saved games" placeholder="⌕  Find a game…" bind:value={filter} />
            <div class="games-list">
              {#each filteredGames as game (game.placeId)}
                <button class:selected={selectedId === game.placeId} class="game-row" onclick={() => selectGame(game.placeId)}>
                  <span class="game-tile">{game.name.slice(0, 1).toUpperCase()}</span>
                  <span class="game-row-name"><strong>{game.name}</strong><small>{game.subplaces.length} subplaces</small></span>
                  <span class="game-chevron">›</span>
                </button>
              {:else}
                <p class="empty-list">No saved games found.</p>
              {/each}
            </div>
            <form class="add-form" onsubmit={event => { event.preventDefault(); addGame(); }}>
              <div class="form-title">+ SAVE A GAME</div>
              <label for="game-name">Experience name</label>
              <input id="game-name" class="input" maxlength="100" placeholder="e.g. Blue Lock Rivals" bind:value={newName} />
              <label for="game-place">Roblox place ID or game URL</label>
              <input id="game-place" class="input" placeholder="e.g. 123456789" bind:value={newPlaceId} />
              <button class="btn primary full" type="submit">Add to library</button>
            </form>
          </div>
          <div class="panel detail-panel">
            {#if selected}
              <div class="game-detail-head">
                <span class="game-tile large">{selected.name.slice(0, 1).toUpperCase()}</span>
                <div class="detail-title"><span class="eyebrow">SAVED EXPERIENCE</span><h2>{selected.name}</h2><p>Place ID {selected.placeId}</p></div>
              </div>
              <div class="detail-actions">
                <button class="btn primary" onclick={() => play(selected.placeId)}>▶ Play experience</button>
                <button class="btn secondary" onclick={() => goToServers(selected.placeId)}>◎ Servers</button>
                <button class="btn danger subtle" onclick={removeGame} aria-label="Remove saved experience">Remove</button>
              </div>
              <div class="sub-header"><h3>Saved subplaces</h3><span>{selected.subplaces.length} PLACES</span></div>
              <div class="subplace-list">
                {#each selected.subplaces as place (place.placeId)}
                  <div class="subplace-row"><div><strong>{place.name}</strong><small>Place ID {place.placeId}</small></div>
                    <div class="row-actions"><button title="Launch subplace" onclick={() => play(place.placeId)}>▶</button><button title="View servers" onclick={() => goToServers(place.placeId)}>◎</button><button title="Remove subplace" onclick={() => removeSubplace(place)}>×</button></div>
                  </div>
                {:else}
                  <div class="empty-state"><div class="empty-mark">◇</div><strong>No subplaces yet</strong><p>Save an individual place to launch it or inspect its servers without searching again.</p></div>
                {/each}
              </div>
              <form class="subplace-form" onsubmit={event => { event.preventDefault(); addSubplace(); }}>
                <div class="form-title">+ ADD SUBPLACE</div>
                <div class="form-row">
                  <input aria-label="Subplace name" class="input" maxlength="100" placeholder="Name (e.g. Pro servers)" bind:value={newSubplaceName} />
                  <input aria-label="Subplace ID" class="input" placeholder="Place ID or URL" bind:value={newSubplaceId} />
                </div>
                <button class="btn secondary" type="submit">Save subplace +</button>
              </form>
            {:else}
              <div class="empty-state big"><div class="empty-mark">◇</div><h3>Build your collection</h3><p>Add an experience on the left to start saving places and exploring servers.</p></div>
            {/if}
          </div>
        </div>
      {:else if tab === "servers"}
        <div class="page-title"><div><span class="eyebrow">LIVE DISCOVERY</span><h1>Public servers</h1><p>Public Roblox server data, requested only when you refresh.</p></div></div>
        <div class="panel server-panel">
          <div class="server-toolbar">
            <label for="server-game">Experience</label>
            <select id="server-game" class="input" value={selectedId ?? ""} onchange={e => selectGame(Number(e.currentTarget.value))}>
              {#if !games.length}<option value="">Save a game first</option>{/if}
              {#each games as game}<option value={game.placeId}>{game.name}</option>{/each}
            </select>
            <label for="server-place">Place</label>
            <select id="server-place" class="input" value={activePlaceId ?? ""} onchange={e => changeActivePlace(Number(e.currentTarget.value))} disabled={!selected}>
              {#each availablePlaces as place}<option value={place.placeId}>{place.name}</option>{/each}
            </select>
            <button class="btn primary" disabled={!activePlaceId || loading} onclick={() => loadServers(false)}>{loading ? "Loading…" : "↻ Refresh"}</button>
          </div>
          <div class="server-disclaimer">Ping shown below is reported by Roblox for each server, <strong>not your personal network ping.</strong> Public server data and pagination can be incomplete or rate-limited.</div>
          <div class="table-head"><span>SERVER</span><span>PLAYERS</span><span>PING</span><span>FPS</span><span></span></div>
          {#each servers as server (server.id)}
            <div class="server-row">
              <span class="server-id"><span class="online-dot"></span>{server.id.slice(0, 8)}…</span>
              <span><strong>{server.playing}</strong><span class="muted"> / {server.maxPlayers}</span></span>
              <span>{server.ping === null ? "—" : Math.round(server.ping) + " ms"}</span>
              <span>{server.fps === null ? "—" : server.fps.toFixed(0)}</span>
              <button class="btn small secondary" onclick={() => activePlaceId && play(activePlaceId, server.id)}>Join ↗</button>
            </div>
          {:else}
            <div class="empty-state server-empty"><div class="empty-mark">◎</div><strong>No server data loaded</strong><p>{games.length ? "Select a saved place and click Refresh to fetch its public servers." : "Add an experience in your library to browse its servers."}</p></div>
          {/each}
          {#if fetchedAt}
            <div class="server-footer"><span>Last refreshed: {fetchedAt.toLocaleTimeString()}</span>
              {#if nextCursor}<button class="btn secondary small" disabled={loading} onclick={() => loadServers(true)}>{loading ? "Loading…" : "Load more ↓"}</button>{/if}
            </div>
          {/if}
        </div>
      {:else}
        <div class="page-title"><div><span class="eyebrow">PREFERENCES & PRIVACY</span><h1>Settings</h1><p>Personalize Thirst without compromising performance.</p></div></div>
        <div class="settings-column">
          <section class="panel settings-panel"><div><h3>Lite Mode</h3><p>Disable decorative gradients and transitions to reduce UI overhead.</p></div><label class="switch"><input type="checkbox" checked={liteMode} onchange={e => setLiteMode(e.currentTarget.checked)} /><span class="slider"></span></label></section>
          <section class="panel settings-panel"><div><h3>Roblox account</h3><p>AppleBlox-style browser login is planned, but not enabled until session isolation and native credential storage are audited. Never paste your .ROBLOSECURITY cookie into the launcher.</p><span class="chip">NOT CONNECTED · COMING LATER</span></div><span class="settings-icon">♙</span></section>
          <section class="panel settings-panel"><div><h3>Network behavior</h3><p>No background polling. Public server requests run from Rust against a fixed Roblox host; there is no Thirst backend or analytics service.</p><span class="chip">LOCAL-FIRST</span></div><span class="settings-icon">◎</span></section>
          <section class="panel settings-panel"><div><h3>Early development</h3><p>Thirst Alpha 0.1.0 · Native macOS and Windows builds require platform testing. Features from AppleBlox/Lunar will be added incrementally.</p></div><span class="settings-icon">◈</span></section>
        </div>
      {/if}
      <footer class="app-footer"><span>THIRST / OPEN SOURCE PROJECT</span><span>BUILT TO STAY OUT OF YOUR GAME'S WAY.</span></footer>
    </div>
  </main>
  {#if toast}
    <div class:bad={toastKind === "bad"} class="toast" role="status"><span>{toastKind === "bad" ? "!" : "✓"}</span>{toast}<button aria-label="Dismiss notification" onclick={() => toast = ""}>×</button></div>
  {/if}
</div>
