<script lang="ts">
  import { onMount } from "svelte";
  import {
    beginRobloxConsent, cancelRobloxLogin, connectedRobloxAccount,
    disconnectRobloxAccount, discoverSubplaces, fetchPublicServers,
    finishRobloxLogin, launchPlace, lookupGame, openRobloxLogin, platformName
  } from "./lib/backend";
  import type { ConnectedAccount } from "./lib/backend";
  import { isTrustedIconUrl, parsePlaceId, readLibrary, readLiteMode, writeLibrary, writeLiteMode } from "./lib/library";
  import type { DiscoveredPlace, GameMetadata, PublicServer, SavedGame, SavedPlace } from "./lib/types";

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
  let toastTimeout: ReturnType<typeof setTimeout> | undefined;
  let loading = false;
  let servers: PublicServer[] = [];
  let cursor: string | null = null;
  let nextCursor: string | null = null;
  let fetchedAt: Date | null = null;
  let addingGame = false;
  let syncingGame = false;
  let discoveryBusy = false;
  let discoveryAttempted = false;
  let discoveredPlaces: DiscoveredPlace[] = [];
  let discoveryCursor: string | null = null;
  let connectedAccount: ConnectedAccount | null = null;
  let checkingAccount = false;
  let accountError = "";
  let loginStep: "warning" | "browser" | null = null;
  let loginBusy = false;
  let readWarning = false;
  let secondsLeft = 15;
  let consentTimer: ReturnType<typeof setInterval> | undefined;

  $: selected = games.find(game => game.placeId === selectedId) || null;
  $: filteredGames = games.filter(game => game.name.toLowerCase().includes(filter.toLowerCase()) ||
    String(game.placeId).includes(filter));
  $: availablePlaces = selected
    ? [
        { placeId: selected.placeId, name: "Main place" },
        ...selected.subplaces,
        ...(activePlaceId && activePlaceId !== selected.placeId &&
          !selected.subplaces.some(place => place.placeId === activePlaceId)
          ? [{ placeId: activePlaceId, name: discoveredPlaces.find(place => place.id === activePlaceId)?.name || "Preview place" }]
          : [])
      ]
    : [];

  onMount(() => {
    games = readLibrary();
    selectedId = games[0]?.placeId ?? null;
    activePlaceId = selectedId;
    liteMode = readLiteMode();
    platformName().then(value => platform = value).catch(() => platform = "Desktop");
    // Prevent a pending notification callback from outliving the desktop window.
    return () => {
      if (toastTimeout) clearTimeout(toastTimeout);
      if (consentTimer) clearInterval(consentTimer);
    };
  });

  async function loadConnectedAccount() {
    if (checkingAccount) return;
    checkingAccount = true;
    accountError = "";
    try {
      connectedAccount = await connectedRobloxAccount();
    } catch (error) {
      connectedAccount = null;
      accountError = String(error);
    } finally {
      checkingAccount = false;
    }
  }

  function stopConsentTimer() {
    if (consentTimer) clearInterval(consentTimer);
    consentTimer = undefined;
  }

  async function beginLogin() {
    if (loginBusy) return;
    loginBusy = true;
    try {
      // Backend sets an independent monotonic timer; changing JS state
      // cannot bypass the minimum 15-second consent gate.
      await beginRobloxConsent();
      loginStep = "warning";
      secondsLeft = 15;
      readWarning = false;
      stopConsentTimer();
      consentTimer = setInterval(() => {
        secondsLeft = Math.max(0, secondsLeft - 1);
        if (secondsLeft === 0) stopConsentTimer();
      }, 1000);
    } catch (error) {
      notice("Could not open cookie consent: " + String(error), "bad");
    } finally {
      loginBusy = false;
    }
  }

  async function openLoginWindow() {
    if (loginBusy || secondsLeft > 0 || !readWarning) return;
    loginBusy = true;
    try {
      await openRobloxLogin(true);
      stopConsentTimer();
      loginStep = "browser";
    } catch (error) {
      notice("Roblox sign-in window failed: " + String(error), "bad");
    } finally {
      loginBusy = false;
    }
  }

  async function completeLogin() {
    if (loginBusy) return;
    loginBusy = true;
    try {
      connectedAccount = await finishRobloxLogin();
      accountError = "";
      loginStep = null;
      notice("Connected to Roblox as @" + connectedAccount.name);
    } catch (error) {
      notice("Sign-in was not completed: " + String(error), "bad");
    } finally {
      loginBusy = false;
    }
  }

  async function cancelLogin() {
    stopConsentTimer();
    loginStep = null;
    try {
      await cancelRobloxLogin();
    } catch {
      notice("Could not close the isolated login window.", "bad");
    }
  }

  async function disconnectAccount() {
    if (loginBusy) return;
    loginBusy = true;
    try {
      await disconnectRobloxAccount();
      connectedAccount = null;
      accountError = "";
      notice("Thirst's saved Roblox session was deleted locally.");
    } catch (error) {
      notice("Unable to disconnect: " + String(error), "bad");
    } finally {
      loginBusy = false;
    }
  }

  function hideBrokenIcon(event: Event) {
    if (event.currentTarget instanceof HTMLImageElement) {
      event.currentTarget.style.display = "none";
    }
  }

  function dismissToast() {
    if (toastTimeout) clearTimeout(toastTimeout);
    toastTimeout = undefined;
    toast = "";
  }

  function notice(message: string, kind: "good" | "bad" = "good") {
    // A new notification must reset the timer for the previous one.
    if (toastTimeout) clearTimeout(toastTimeout);
    toast = message;
    toastKind = kind;
    toastTimeout = setTimeout(() => {
      toast = "";
      toastTimeout = undefined;
    }, kind === "bad" ? 8000 : 3500);
  }

  function persist() {
    try { writeLibrary(games); } catch { notice("Couldn't save to local storage.", "bad"); }
  }

  async function addGame() {
    const inputId = parsePlaceId(newPlaceId);
    const alias = newName.trim().slice(0, 100);
    if (!inputId) return notice("Enter a valid Roblox place ID or game URL.", "bad");
    if (addingGame) return;
    if (games.length >= 200) return notice("Library limit reached.", "bad");
    addingGame = true;
    try {
      let metadata: GameMetadata | null = null;
      try {
        metadata = await lookupGame(inputId);
      } catch (error) {
        // Manual bookmarks should continue working if public metadata is unavailable.
        if (!alias) return notice("Roblox lookup failed. Enter a name to save manually. " + String(error), "bad");
      }
      const gameId = metadata?.rootPlaceId ?? inputId;
      if (games.some(game => game.placeId === gameId)) {
        return notice("This experience is already saved. Add its other places in the subplace section.", "bad");
      }
      const game: SavedGame = {
        placeId: gameId,
        name: alias || metadata?.name || "Unnamed experience",
        subplaces: inputId !== gameId ? [{ placeId: inputId, name: "Originally selected place" }] : [],
        addedAt: Date.now(),
        universeId: metadata?.universeId,
        rootPlaceId: metadata?.rootPlaceId,
        iconUrl: isTrustedIconUrl(metadata?.iconUrl) ? metadata.iconUrl : undefined
      };
      games = [...games, game];
      persist();
      selectGame(gameId);
      newName = "";
      newPlaceId = "";
      notice(metadata ? "Experience and official Roblox icon saved." : "Saved manually; metadata can be synced later.");
    } finally {
      addingGame = false;
    }
  }

  async function syncSelectedGame() {
    const id = selectedId;
    if (!id || syncingGame) return;
    syncingGame = true;
    try {
      const metadata = await lookupGame(id);
      if (selectedId !== id) return;
      games = games.map(game => game.placeId === id ? {
        ...game, universeId: metadata.universeId, rootPlaceId: metadata.rootPlaceId,
        iconUrl: isTrustedIconUrl(metadata.iconUrl) ? metadata.iconUrl : game.iconUrl
      } : game);
      persist();
      notice("Official game icon and metadata updated.");
    } catch (error) {
      notice("Could not sync metadata: " + String(error), "bad");
    } finally {
      syncingGame = false;
    }
  }

  async function discoverPlaces(loadMore = false) {
    const id = selectedId;
    const selectedGame = games.find(game => game.placeId === id);
    if (!id || !selectedGame || discoveryBusy) return;
    if (loadMore && !discoveryCursor) return;
    discoveryBusy = true;
    try {
      let universeId = selectedGame.universeId;
      let rootPlaceId = selectedGame.rootPlaceId ?? selectedGame.placeId;
      if (!universeId) {
        const metadata = await lookupGame(selectedGame.placeId);
        if (selectedId !== id) return;
        universeId = metadata.universeId;
        rootPlaceId = metadata.rootPlaceId;
        games = games.map(game => game.placeId === id ? {
          ...game, universeId: metadata.universeId, rootPlaceId: metadata.rootPlaceId,
          iconUrl: isTrustedIconUrl(metadata.iconUrl) ? metadata.iconUrl : game.iconUrl
        } : game);
        persist();
      }
      const result = await discoverSubplaces(universeId, loadMore ? discoveryCursor : null);
      if (selectedId !== id) return;
      const incoming = result.data.filter(place =>
        place.id !== selectedGame.placeId && place.id !== rootPlaceId);
      discoveredPlaces = loadMore ?
        [...discoveredPlaces, ...incoming.filter(place => !discoveredPlaces.some(old => old.id === place.id))] :
        incoming;
      discoveryCursor = result.nextPageCursor;
      discoveryAttempted = true;
    } catch (error) {
      if (selectedId === id) {
        notice("Subplaces could not be discovered publicly. You can still add them manually. " + String(error), "bad");
      }
    } finally {
      discoveryBusy = false;
    }
  }

  function saveDiscoveredPlace(place: DiscoveredPlace) {
    if (!selected || place.id === selected.placeId ||
        selected.subplaces.some(item => item.placeId === place.id)) return;
    if (selected.subplaces.length >= 100) return notice("Subplace limit reached.", "bad");
    const id = selected.placeId;
    games = games.map(game => game.placeId === id ? {
      ...game, subplaces: [...game.subplaces, {
        placeId: place.id, name: place.name.slice(0, 100) || "Unnamed place"
      }]
    } : game);
    persist();
    notice("Subplace added to your library.");
  }

  function clearDiscovery() {
    discoveredPlaces = [];
    discoveryCursor = null;
    discoveryAttempted = false;
  }

  function selectGame(id: number) {
    selectedId = id;
    clearDiscovery();
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
      <button class:active={tab === "settings"} onclick={() => { tab = "settings"; void loadConnectedAccount(); }}><span class="nav-icon">⚙</span> Settings</button>
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
              <button class="btn primary" onclick={() => tab = "library"}>{games.length ? "▦ Open your library" : "+ Add your first game"}</button>
              <button class="btn secondary" onclick={() => tab = "servers"}>Browse servers →</button>
            </div>
          </div>
          <div class="hero-art" aria-hidden="true"><div class="orb orb-outer"><div class="orb orb-inner">T</div></div></div>
        </div>
        <div class="stats">
          <div class="stat"><small>SAVED EXPERIENCES</small><strong>{games.length.toString().padStart(2, "0")}</strong><span>In your personal library</span></div>
          <div class="stat"><small>BOOKMARKED SUBPLACES</small><strong>{games.reduce((n, g) => n + g.subplaces.length, 0).toString().padStart(2, "0")}</strong><span>Quick launch shortcuts</span></div>
          <div class="stat"><small>BACKGROUND REFRESH</small><strong>OFF</strong><span>Servers load only on request</span></div>
        </div>
        {#if games.length}
          <div class="section-heading"><div><span class="eyebrow">ONE-CLICK LAUNCH</span><h2>Your favorites</h2></div><button class="link-button" onclick={() => tab = "library"}>Manage library ↗</button></div>
          <div class="quick-launch-grid">
            {#each games.slice(0, 12) as game (game.placeId)}
              <div class="quick-game">
                <button class="quick-game-main" onclick={() => play(game.placeId)} title={"Launch " + game.name} aria-label={"Launch " + game.name}>
                  <span class="game-tile quick-game-icon">{game.name.slice(0, 1).toUpperCase()}
                    {#if isTrustedIconUrl(game.iconUrl)}<img src={game.iconUrl} alt="" loading="lazy" decoding="async" onerror={hideBrokenIcon} />{/if}
                  </span>
                  <span class="quick-game-title">{game.name}</span>
                </button>
                <button class="quick-game-server" title={"Browse " + game.name + " servers"} aria-label={"Browse " + game.name + " servers"} onclick={() => { selectGame(game.placeId); tab = "servers"; }}>◎</button>
              </div>
            {/each}
          </div>
        {/if}
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
                  <span class="game-tile">{game.name.slice(0, 1).toUpperCase()}
                    {#if isTrustedIconUrl(game.iconUrl)}<img src={game.iconUrl} alt="" loading="lazy" decoding="async" onerror={hideBrokenIcon} />{/if}
                  </span>
                  <span class="game-row-name"><strong>{game.name}</strong><small>{game.subplaces.length} subplaces</small></span>
                  <span class="game-chevron">›</span>
                </button>
              {:else}
                <p class="empty-list">No saved games found.</p>
              {/each}
            </div>
            <form class="add-form" onsubmit={event => { event.preventDefault(); addGame(); }}>
              <div class="form-title">+ SAVE A GAME</div>
              <label for="game-name">Custom display name (optional)</label>
              <input id="game-name" class="input" maxlength="100" placeholder="Otherwise use the official Roblox name" bind:value={newName} />
              <label for="game-place">Roblox place ID or game URL</label>
              <input id="game-place" class="input" placeholder="e.g. 123456789" bind:value={newPlaceId} />
              <button class="btn primary full" type="submit" disabled={addingGame}>{addingGame ? "Looking up Roblox…" : "Add to library"}</button>
              <p class="form-hint">Thirst fetches the official name and icon only when you add a game. If Roblox is unavailable, you can still save it with a custom name.</p>
            </form>
          </div>
          <div class="panel detail-panel">
            {#if selected}
              <div class="game-detail-head">
                <span class="game-tile large">{selected.name.slice(0, 1).toUpperCase()}
                  {#if isTrustedIconUrl(selected.iconUrl)}<img src={selected.iconUrl} alt="" loading="lazy" decoding="async" onerror={hideBrokenIcon} />{/if}
                </span>
                <div class="detail-title"><span class="eyebrow">SAVED EXPERIENCE</span><h2>{selected.name}</h2><p>Place ID {selected.placeId}</p></div>
              </div>
              <div class="detail-actions">
                <button class="btn primary" onclick={() => play(selected.placeId)}>▶ Play experience</button>
                <button class="btn secondary" onclick={() => goToServers(selected.placeId)}>◎ Servers</button>
                <button class="btn secondary" disabled={syncingGame} onclick={syncSelectedGame}>{syncingGame ? "Syncing…" : "↻ Update icon"}</button>
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
              <section class="discovery-box">
                <div class="discovery-heading"><div><h3>Explore subplaces</h3><p>Retrieve places Roblox publicly lists for this experience. No login required.</p></div>
                  <button class="btn secondary small" onclick={() => discoverPlaces(false)} disabled={discoveryBusy}>{discoveryBusy ? "Searching…" : "Discover places"}</button>
                </div>
                {#if discoveryAttempted}
                  <div class="discover-results">
                    {#each discoveredPlaces as place (place.id)}
                      <div class="discovered-place"><div><strong>{place.name}</strong><small>ID {place.id}</small></div>
                        <div class="row-actions">
                          {#if selected.subplaces.some(item => item.placeId === place.id)}
                            <span class="saved-label">SAVED</span>
                          {:else}
                            <button class="save-place" title={"Save " + place.name} aria-label={"Save " + place.name} onclick={() => saveDiscoveredPlace(place)}>+ Save</button>
                          {/if}
                          <button title={"Open server browser for " + place.name} onclick={() => goToServers(place.id)}>◎</button>
                        </div>
                      </div>
                    {:else}
                      <p class="form-hint">No additional public subplaces were returned on this page. Private or inaccessible places won't appear.</p>
                    {/each}
                    {#if discoveryCursor}
                      <button class="btn secondary small" disabled={discoveryBusy} onclick={() => discoverPlaces(true)}>{discoveryBusy ? "Loading…" : "Load more places ↓"}</button>
                    {/if}
                  </div>
                {/if}
              </section>
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
          <section class="panel settings-panel account-settings">
            <div class="account-info">
              <h3>Roblox account</h3>
              <p>Optional browser-based session connection. Your session cookie is read only by native Rust and stored in your OS credential vault, never in Thirst's UI, SQLite, or our server.</p>
              {#if connectedAccount}
                <div class="connected-account">
                  <span class="connected-indicator"></span>
                  <div><strong>{connectedAccount.displayName} (@{connectedAccount.name})</strong><small>Roblox user ID: {connectedAccount.id}</small></div>
                </div>
              {:else if checkingAccount}
                <span class="chip">CHECKING ACCOUNT…</span>
              {:else if accountError}
                <p class="account-error">Saved account could not be verified: {accountError}</p>
              {:else}
                <span class="chip">NO VERIFIED ACCOUNT</span>
              {/if}
              <p class="account-footnote">This is an experimental full-session login. A stolen cookie can compromise your Roblox account. Local disconnect does not revoke the Roblox session on Roblox's servers.</p>
              <div class="account-buttons">
                <button class="btn primary" onclick={beginLogin} disabled={loginBusy}>Connect Roblox account</button>
                <button class="btn secondary" onclick={loadConnectedAccount} disabled={checkingAccount}>↻ Check status</button>
                {#if connectedAccount || accountError}
                  <button class="btn danger subtle" onclick={disconnectAccount} disabled={loginBusy}>Disconnect / delete local credential</button>
                {/if}
              </div>
            </div>
            <span class="settings-icon">♙</span>
          </section>
          <section class="panel settings-panel"><div><h3>Network behavior</h3><p>No background polling. On-demand requests use fixed Roblox API hosts in Rust; approved game icons load from Roblox CDN. No Thirst backend or analytics service.</p><span class="chip">LOCAL-FIRST</span></div><span class="settings-icon">◎</span></section>
          <section class="panel settings-panel"><div><h3>Early development</h3><p>Thirst Alpha 0.3 (experimental authentication). Test with a separate, low-value Roblox account before using your main account.</p></div><span class="settings-icon">◈</span></section>
        </div>
      {/if}
      <footer class="app-footer"><span>THIRST / OPEN SOURCE PROJECT</span><span>BUILT TO STAY OUT OF YOUR GAME'S WAY.</span></footer>
    </div>
  </main>
  {#if loginStep}
    <div class="consent-overlay">
      <section role="dialog" aria-modal="true" aria-labelledby="login-warning-title" class="consent-dialog">
        <div class="consent-topline"><span class="consent-alert-icon">!</span><span>THIRST ACCOUNT CONNECTION · HIGH-RISK PERMISSION</span></div>
        {#if loginStep === "warning"}
          <h2 id="login-warning-title">Read before connecting your Roblox account.</h2>
          <p class="consent-lead">Thirst will access your <strong>.ROBLOSECURITY session cookie</strong> after you sign in on the official Roblox website. This cookie can grant full access to your account.</p>
          <div class="consent-disclosure">
            <div><strong>What Thirst will access</strong><p>A Roblox session cookie from an isolated sign-in window. A stolen copy may let someone access your account without your password or 2-step verification.</p></div>
            <div><strong>Where it will be stored</strong><p>Locally using macOS Keychain or Windows Credential Manager, not in browser localStorage, project files, logs or cloud storage.</p></div>
            <div><strong>Where it will be sent</strong><p>Only to the fixed Roblox verification endpoint when you choose to connect or check your session. No Thirst, RoValra or Discord server receives it.</p></div>
            <div><strong>Important limitations</strong><p>Native local malware or a compromised launcher build could still steal a cookie. This is an experimental feature and is not independently security-audited.</p></div>
          </div>
          <label class="consent-confirm">
            <input type="checkbox" bind:checked={readWarning} />
            <span>I understand that Thirst will read and store my Roblox session cookie locally, and that I can choose not to connect.</span>
          </label>
          <div class="consent-footer">
            <button class="btn secondary" onclick={cancelLogin}>Cancel — use Thirst as guest</button>
            <button class="btn primary" onclick={openLoginWindow} disabled={secondsLeft > 0 || !readWarning || loginBusy}>
              {secondsLeft > 0 ? "Read warning (" + secondsLeft + "s)" : "Open official Roblox login"}
            </button>
          </div>
          <p class="consent-endnote">15-second review is enforced by the native app as well as this screen. It does not eliminate the security risk.</p>
        {:else}
          <h2 id="login-warning-title">Sign in on the official Roblox page.</h2>
          <p class="consent-lead">The isolated browser window opened to <strong>https://www.roblox.com/login</strong>. Complete Roblox sign-in there, then return here and click <strong>Finish connection</strong>.</p>
          <div class="consent-disclosure">
            <div><strong>No copying cookies</strong><p>Do not paste your .ROBLOSECURITY value anywhere. Thirst reads it from its own isolated browser session only when you finish.</p></div>
            <div><strong>Use a test account</strong><p>Because authentication is still experimental, please test it with an account you would be comfortable revoking.</p></div>
          </div>
          <div class="consent-footer">
            <button class="btn secondary" onclick={cancelLogin} disabled={loginBusy}>Cancel sign-in</button>
            <button class="btn primary" onclick={completeLogin} disabled={loginBusy}>{loginBusy ? "Verifying with Roblox…" : "Finish connection"}</button>
          </div>
        {/if}
      </section>
    </div>
  {/if}
  {#if toast}
    <div class:bad={toastKind === "bad"} class="toast" role="status"><span>{toastKind === "bad" ? "!" : "✓"}</span>{toast}<button aria-label="Dismiss notification" onclick={dismissToast}>×</button></div>
  {/if}
</div>
