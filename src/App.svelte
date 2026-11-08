<script lang="ts">
  import { onMount } from "svelte";
  import {
    beginRobloxConsent, cancelRobloxLogin, connectedRobloxAccount,
    disconnectRobloxAccount, discoverSubplaces, fetchPublicServers, fetchServerAvatarPreview, fetchServerAvatarPreviews,
    finishRobloxLogin, launchPlace, lookupGame, lookupRoValraRegions, openRobloxLogin, platformName, pollRobloxLoginStatus
  } from "./lib/backend";
  import type { ConnectedAccount } from "./lib/backend";
  import { isTrustedIconUrl, parsePlaceId, readAutoAvatarPreviews, readLibrary, readLibrarySort, readLiteMode, writeAutoAvatarPreviews, writeLibrary, writeLibrarySort, writeLiteMode } from "./lib/library";
  import type { DiscoveredPlace, GameMetadata, PublicServer, SavedGame, SavedPlace } from "./lib/types";
  import { filterAndSortServers, isValidServerId } from "./lib/servers";
  import type { ServerSort } from "./lib/servers";
  import type { LibrarySort } from "./lib/library";
  import {
    REGION_OPTIONS, providerRegionLabel, readRegionReports, regionLabel, reportForServer,
    withRegionReport, withoutRegionReport, writeRegionReports
  } from "./lib/regions";
  import type { LocalRegionReport, ProviderRegion, RegionCode, RegionFilter } from "./lib/regions";

  type Tab = "home" | "library" | "servers" | "settings";
  type QuickLaunchItem = {
    placeId: number;
    parentPlaceId: number;
    name: string;
    parentName: string;
    iconUrl?: string;
    isSubplace: boolean;
  };
  let tab: Tab = "home";
  let games: SavedGame[] = [];
  let selectedId: number | null = null;
  let activePlaceId: number | null = null;
  let newName = "";
  let newPlaceId = "";
  let newSubplaceName = "";
  let newSubplaceId = "";
  let filter = "";
  let librarySort: LibrarySort = "saved";
  let liteMode = false;
  let platform = "Desktop";
  let toast = "";
  let toastKind: "good" | "bad" = "good";
  let toastTimeout: ReturnType<typeof setTimeout> | undefined;
  let loading = false;
  let servers: PublicServer[] = [];
  let serverSort: ServerSort = "default";
  let serverHideFull = false;
  let serverMaxPing = "any";
  let serverSearch = "";
  let regionFilter: RegionFilter = "any";
  let regionReports: LocalRegionReport[] = [];
  // Provider access is a session-only opt-in. It never reads Roblox credentials
  // and is not used until explicitly enabled by the user.
  let rovalraEnabled = false;
  let rovalraLoading = false;
  let rovalraError = "";
  let providerRegions: Record<string, ProviderRegion> = {};
  let editingRegionId: string | null = null;
  let regionDraft: RegionCode | "" = "";
  let serverLoadError = "";
  let autoAvatarEnabled = true;
  // Deliberately NOT persisted. Full-session credentials require fresh opt-in per app run.
  let useAuthenticatedAvatarRequests = false;
  let bulkPreviewBusy = false;
  let bulkPreviewRequested: string[] = [];
  let bulkPreviewError = "";
  let bulkPreviewVersion = 0;
  let nextServerRequestAt = 0;
  let expandedPreviewId: string | null = null;
  let previewLoadingId: string | null = null;
  let avatarPreviews: Record<string, string[]> = {};
  let avatarErrors: Record<string, string> = {};
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
  // A single account-status lookup per app session avoids repeatedly
  // requesting Keychain permission when navigating back to Settings.
  // Users can still explicitly refresh with Check status.
  let accountStatusLoaded = false;
  let accountError = "";
  let loginStep: "warning" | "browser" | null = null;
  let loginBusy = false;
  let readWarning = false;
  let secondsLeft = 15;
  let consentTimer: ReturnType<typeof setInterval> | undefined;
  let nativeLoginTimer: ReturnType<typeof setInterval> | undefined;
  let nativeLoginCheckInProgress = false;
  let hoverFrame = 0;
  let hoverPending: { target: HTMLElement; x: number; y: number } | null = null;

  $: selected = games.find(game => game.placeId === selectedId) || null;
  $: visibleServers = filterAndSortServers(servers, {
    sort: serverSort, hideFull: serverHideFull,
    maxPing: serverMaxPing === "any" ? null : Number(serverMaxPing),
    search: serverSearch, region: regionFilter
  }, regionReports, activePlaceId, providerRegions);

  $: filteredGames = games
    .filter(game => game.name.toLowerCase().includes(filter.toLowerCase()) ||
      String(game.placeId).includes(filter))
    .slice()
    .sort((a, b) => {
      if (librarySort === "name") return a.name.localeCompare(b.name);
      if (librarySort === "newest") return b.addedAt - a.addedAt;
      if (librarySort === "oldest") return a.addedAt - b.addedAt;
      return 0; // preserve saved order, including existing bookmarks
    });
  $: quickLaunchItems = games.flatMap((game): QuickLaunchItem[] => [
    {
      placeId: game.placeId, parentPlaceId: game.placeId,
      name: game.name, parentName: game.name,
      iconUrl: game.iconUrl, isSubplace: false
    },
    ...game.subplaces.filter(place => place.pinned === true).map((place): QuickLaunchItem => ({
      placeId: place.placeId, parentPlaceId: game.placeId,
      name: place.name, parentName: game.name,
      iconUrl: game.iconUrl, isSubplace: true
    }))
  ]).slice(0, 24);
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
    regionReports = readRegionReports();
    selectedId = games[0]?.placeId ?? null;
    activePlaceId = selectedId;
    liteMode = readLiteMode();
    librarySort = readLibrarySort();
    autoAvatarEnabled = readAutoAvatarPreviews();
    platformName().then(value => platform = value).catch(() => platform = "Desktop");
    // Prevent a pending notification callback from outliving the desktop window.
    return () => {
      if (toastTimeout) clearTimeout(toastTimeout);
      if (consentTimer) clearInterval(consentTimer);
      if (nativeLoginTimer) clearInterval(nativeLoginTimer);
      if (hoverFrame) cancelAnimationFrame(hoverFrame);
    };
  });

  async function loadConnectedAccount() {
    if (checkingAccount) return;
    checkingAccount = true;
    accountError = "";
    try {
      connectedAccount = await connectedRobloxAccount();
      if (connectedAccount && tab === "servers") void loadAutomaticAvatars();
    } catch (error) {
      connectedAccount = null;
      accountError = String(error);
    } finally {
      checkingAccount = false;
      accountStatusLoaded = true;
    }
  }

  function stopConsentTimer() {
    if (consentTimer) clearInterval(consentTimer);
    consentTimer = undefined;
  }

  function stopNativeLoginPolling() {
    if (nativeLoginTimer) clearInterval(nativeLoginTimer);
    nativeLoginTimer = undefined;
  }

  async function checkNativeLogin() {
    if (nativeLoginCheckInProgress || loginBusy || loginStep !== "browser") return;
    nativeLoginCheckInProgress = true;
    try {
      const status = await pollRobloxLoginStatus();
      if (status === "ready") {
        stopNativeLoginPolling();
        await completeLogin();
      } else if (status === "cancelled") {
        stopNativeLoginPolling();
        loginStep = null;
        await cancelRobloxLogin();
        notice("Roblox sign-in was cancelled.");
      }
    } catch {
      // Transient Keychain or process status failures should not close login.
      // The user can still click Finish connection manually.
    } finally {
      nativeLoginCheckInProgress = false;
    }
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
      if (platform === "macOS") {
        stopNativeLoginPolling();
        nativeLoginTimer = setInterval(() => { void checkNativeLogin(); }, 1500);
      }
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
      if (tab === "servers") void loadAutomaticAvatars();
      accountStatusLoaded = true;
      accountError = "";
      loginStep = null;
      stopNativeLoginPolling();
      notice("Connected to Roblox as @" + connectedAccount.name);
    } catch (error) {
      notice("Sign-in was not completed: " + String(error), "bad");
    } finally {
      loginBusy = false;
    }
  }

  async function cancelLogin() {
    stopConsentTimer();
    stopNativeLoginPolling();
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
      useAuthenticatedAvatarRequests = false;
      bulkPreviewVersion++;
      avatarPreviews = {};
      accountStatusLoaded = true;
      accountError = "";
      notice("Thirst's saved Roblox session was deleted locally.");
    } catch (error) {
      notice("Unable to disconnect: " + String(error), "bad");
    } finally {
      loginBusy = false;
    }
  }

  function moveHoverLight(event: PointerEvent) {
    // Work occurs only while the pointer moves, at most once per animation frame.
    if (liteMode || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const target = event.currentTarget;
    if (!(target instanceof HTMLElement)) return;
    const bounds = target.getBoundingClientRect();
    hoverPending = { target, x: event.clientX - bounds.left, y: event.clientY - bounds.top };
    if (hoverFrame) return;
    hoverFrame = requestAnimationFrame(() => {
      if (hoverPending) {
        hoverPending.target.style.setProperty("--glow-x", hoverPending.x + "px");
        hoverPending.target.style.setProperty("--glow-y", hoverPending.y + "px");
        hoverPending = null;
      }
      hoverFrame = 0;
    });
  }

  function togglePin(place: SavedPlace) {
    if (!selected) return;
    const parentId = selected.placeId;
    const wasPinned = place.pinned === true;
    games = games.map(game => game.placeId === parentId ? {
      ...game,
      subplaces: game.subplaces.map(item =>
        item.placeId === place.placeId ? { ...item, pinned: !wasPinned } : item
      )
    } : game);
    persist();
    notice(wasPinned ? "Subplace removed from home favorites." : "Subplace pinned to home favorites.");
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
    serverLoadError = "";
    editingRegionId = null;
    regionDraft = "";
    providerRegions = {};
    rovalraError = "";
    expandedPreviewId = null;
    previewLoadingId = null;
    avatarPreviews = {};
    avatarErrors = {};
    bulkPreviewVersion++;
    bulkPreviewRequested = [];
    bulkPreviewError = "";
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
    serverLoadError = "";
    editingRegionId = null;
    regionDraft = "";
    providerRegions = {};
    rovalraError = "";
    expandedPreviewId = null;
    previewLoadingId = null;
    avatarPreviews = {};
    avatarErrors = {};
    bulkPreviewVersion++;
    bulkPreviewRequested = [];
    bulkPreviewError = "";
  }

  async function copyServerId(serverId: string) {
    if (!isValidServerId(serverId)) return;
    try {
      await navigator.clipboard.writeText(serverId);
      notice("Roblox server ID copied to clipboard.");
    } catch {
      notice("Clipboard access is unavailable. Hover the ID to see it in full.", "bad");
    }
  }

  function setLibrarySort(sort: LibrarySort) {
    librarySort = sort;
    try { writeLibrarySort(sort); }
    catch { notice("Could not save library sort preference.", "bad"); }
  }

  function setRoValraEnabled(enabled: boolean) {
    rovalraEnabled = enabled;
    rovalraError = "";
    providerRegions = {};
    if (enabled) {
      notice("RoValra API enabled for this run. Only public place and server IDs are shared. No cookies.");
      if (activePlaceId && servers.length && !rovalraLoading) void loadRoValraRegions();
    } else {
      notice("RoValra lookups disabled. Your local region reports remain available.");
    }
  }

  async function loadRoValraRegions() {
    if (!rovalraEnabled || rovalraLoading || !activePlaceId || !servers.length) return;
    const placeId = activePlaceId;
    const ids = servers.slice(0, 25).map(server => server.id).filter(isValidServerId);
    if (!ids.length) return;
    rovalraLoading = true;
    rovalraError = "";
    try {
      const response = await lookupRoValraRegions(placeId, ids);
      if (!rovalraEnabled || activePlaceId !== placeId) return;
      const allowed = new Set(ids.map(id => id.toLowerCase()));
      const byId: Record<string, ProviderRegion> = {};
      for (const item of response) {
        const id = item.serverId.toLowerCase();
        if (allowed.has(id) && item.source === "rovalra") byId[id] = item;
      }
      providerRegions = { ...providerRegions, ...byId };
      notice("RoValra reported regions for " + Object.keys(byId).length + " loaded servers.");
    } catch (error) {
      if (rovalraEnabled && activePlaceId === placeId) {
        rovalraError = String(error);
        if (/did not permit|disabled region lookup/i.test(rovalraError)) {
          rovalraEnabled = false;
          providerRegions = {};
        }
      }
    } finally {
      rovalraLoading = false;
    }
  }

  function regionReportFor(serverId: string): LocalRegionReport | null {
    return reportForServer(regionReports, activePlaceId, serverId);
  }

  function displayServerRegion(serverId: string): string {
    const report = regionReportFor(serverId);
    return report ? regionLabel(report.region) : providerRegionLabel(providerRegions[serverId.toLowerCase()]);
  }

  function editServerRegion(serverId: string) {
    if (editingRegionId === serverId) {
      editingRegionId = null;
      regionDraft = "";
      return;
    }
    editingRegionId = serverId;
    regionDraft = regionReportFor(serverId)?.region ?? "";
  }

  function saveServerRegion(serverId: string) {
    if (!activePlaceId || !isValidServerId(serverId) || !regionDraft) return;
    try {
      regionReports = writeRegionReports(
        withRegionReport(regionReports, activePlaceId, serverId, regionDraft)
      );
      editingRegionId = null;
      regionDraft = "";
      notice("Your local region report was saved. It expires after two hours and is not verified.");
    } catch {
      notice("Could not store the local server-region report.", "bad");
    }
  }

  function clearServerRegion(serverId: string) {
    if (!activePlaceId) return;
    try {
      regionReports = writeRegionReports(withoutRegionReport(regionReports, activePlaceId, serverId));
      editingRegionId = null;
      regionDraft = "";
      notice("Local region report cleared.");
    } catch {
      notice("Could not remove the local region report.", "bad");
    }
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
    if (!activePlaceId || loading || (loadMore && !nextCursor)) return;
    // Respect Roblox's public endpoint limits without background timers.
    // This only throttles requests from this app, not all devices on the IP.
    const waitSeconds = Math.ceil((nextServerRequestAt - Date.now()) / 1000);
    if (waitSeconds > 0) {
      notice("Server requests are cooling down. Try again in " + waitSeconds + "s.");
      return;
    }
    const requestedPlace = activePlaceId;
    nextServerRequestAt = Date.now() + 8000;
    loading = true;
    serverLoadError = "";
    try {
      const result = await fetchPublicServers(
        requestedPlace, loadMore ? nextCursor : null,
        useAuthenticatedAvatarRequests && connectedAccount !== null
      );
      // If the selected subplace changes while a request is outstanding, ignore
      // the old result rather than mixing server lists from two places.
      if (activePlaceId !== requestedPlace) return;
      if (!loadMore) {
        // Thumbnails describe a momentary player sample, not stable identities.
        // Refreshing servers invalidates those previews.
        expandedPreviewId = null;
        previewLoadingId = null;
        avatarPreviews = {};
        avatarErrors = {};
        providerRegions = {};
        rovalraError = "";
        bulkPreviewVersion++;
        bulkPreviewRequested = [];
      }
      servers = loadMore ?
        [...servers, ...result.data.filter(server => !servers.some(old => old.id === server.id))] :
        result.data;
      cursor = loadMore ? nextCursor : null;
      nextCursor = result.nextPageCursor;
      fetchedAt = new Date();
      // Remove expired reports when a new public server sample is requested.
      regionReports = regionReports.filter(entry => {
        const age = Date.now() - entry.observedAt;
        return age >= 0 && age < 2 * 60 * 60 * 1000;
      });
      if (!loadMore && connectedAccount) void loadAutomaticAvatars();
      if (!loadMore && rovalraEnabled && !rovalraLoading) void loadRoValraRegions();
      if (!servers.length) notice("No public servers returned for this place.");
    } catch (error) {
      if (activePlaceId === requestedPlace) {
        serverLoadError = String(error);
        if (/rate.limit|429/i.test(serverLoadError)) {
          nextServerRequestAt = Date.now() + 45000;
        }
        notice("Could not fetch servers: " + serverLoadError, "bad");
      }
    } finally {
      loading = false;
    }
  }

  async function loadAutomaticAvatars() {
    if (!connectedAccount || !autoAvatarEnabled || liteMode || !activePlaceId || bulkPreviewBusy) return;
    // One batch: max 8 servers, 5 anonymous avatars each, per manual refresh.
    // Public thumbnails do not use cookies or identify players.
    const placeId = activePlaceId;
    const ids = servers.slice(0, 8).map(server => server.id).filter(isValidServerId)
      .filter(id => !Object.prototype.hasOwnProperty.call(avatarPreviews, id));
    if (!ids.length) return;
    const version = ++bulkPreviewVersion;
    bulkPreviewBusy = true;
    bulkPreviewRequested = ids;
    bulkPreviewError = "";
    try {
      const result = await fetchServerAvatarPreviews(placeId, ids, useAuthenticatedAvatarRequests && connectedAccount !== null);
      if (activePlaceId !== placeId || version !== bulkPreviewVersion) return;
      const cleaned: Record<string, string[]> = {};
      for (const id of ids) {
        cleaned[id] = (result[id] ?? []).filter(isTrustedIconUrl).slice(0, 5);
      }
      avatarPreviews = { ...avatarPreviews, ...cleaned };
    } catch (error) {
      if (activePlaceId === placeId && version === bulkPreviewVersion) {
        bulkPreviewError = String(error);
      }
    } finally {
      const wasSuperseded = version !== bulkPreviewVersion;
      bulkPreviewBusy = false;
      bulkPreviewRequested = [];
      // If the user changed place or refreshed mid-request, load the new
      // server batch once the previous network request has settled.
      if (wasSuperseded && tab === "servers") void loadAutomaticAvatars();
    }
  }

  function setAuthenticatedAvatars(enabled: boolean) {
    if (enabled && !connectedAccount) {
      notice("Verify your Roblox account in Settings before using its saved session.", "bad");
      return;
    }
    useAuthenticatedAvatarRequests = enabled;
    // Never persist session-backed request permission. Re-authorize after restart.
    avatarPreviews = {};
    avatarErrors = {};
    bulkPreviewError = "";
    bulkPreviewVersion++;
    bulkPreviewRequested = [];
    notice(enabled
      ? "Authenticated Roblox server/thumbnail requests enabled for this app session. Click Refresh to test."
      : "Authenticated avatar requests disabled. Public mode will be used.");
  }

  function setAutoAvatarEnabled(enabled: boolean) {
    autoAvatarEnabled = enabled;
    try { writeAutoAvatarPreviews(enabled); }
    catch { notice("Could not save avatar preferences.", "bad"); }
    if (enabled && tab === "servers") void loadAutomaticAvatars();
  }

  async function toggleServerPreview(serverId: string) {
    const placeId = activePlaceId;
    if (!placeId || !isValidServerId(serverId)) return;
    if (expandedPreviewId === serverId) {
      expandedPreviewId = null;
      return;
    }
    expandedPreviewId = serverId;
    if (Object.prototype.hasOwnProperty.call(avatarPreviews, serverId) ||
        avatarErrors[serverId] || previewLoadingId === serverId) return;
    previewLoadingId = serverId;
    try {
      const urls = await fetchServerAvatarPreview(placeId, serverId, useAuthenticatedAvatarRequests && connectedAccount !== null);
      if (activePlaceId !== placeId) return;
      avatarPreviews = { ...avatarPreviews, [serverId]: urls.filter(isTrustedIconUrl).slice(0, 5) };
    } catch (error) {
      if (activePlaceId === placeId) {
        avatarErrors = { ...avatarErrors, [serverId]: String(error) };
      }
    } finally {
      if (previewLoadingId === serverId) previewLoadingId = null;
    }
  }

  function setLiteMode(enabled: boolean) {
    liteMode = enabled;
    try { writeLiteMode(enabled); } catch { notice("Couldn't save preference.", "bad"); }
  }

  function enterServerBrowser() {
    tab = "servers";
    // Verify once on visiting Servers. Don't read Keychain just to download
    // public thumbnails, and don't prompt on every application startup.
    if (!accountStatusLoaded && !checkingAccount) void loadConnectedAccount();
  }

  function goToServers(id?: number) {
    if (id) changeActivePlace(id);
    enterServerBrowser();
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
      <button class:active={tab === "servers"} onclick={enterServerBrowser}><span class="nav-icon">◎</span> Server browser</button>
    </nav>
    <span class="section-label section-spaced">PREFERENCES</span>
    <nav aria-label="Preferences">
      <button class:active={tab === "settings"} onclick={() => { tab = "settings"; if (!accountStatusLoaded) void loadConnectedAccount(); }}><span class="nav-icon">⚙</span> Settings</button>
    </nav>
    <div class="sidebar-bottom">
      <div class="sidebar-note"><span class="note-indicator"></span><span>Local-first · No account needed</span></div>
      <div class="version">THIRST ALPHA <span>0.3.0</span></div>
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
              <button class="btn secondary" onclick={enterServerBrowser}>Browse servers →</button>
            </div>
          </div>
          <div class="hero-art" aria-hidden="true"><div class="orb orb-outer"><div class="orb orb-inner">T</div></div></div>
        </div>
        <div class="stats">
          <div class="stat hover-glow" onpointermove={moveHoverLight}><small>SAVED EXPERIENCES</small><strong>{games.length.toString().padStart(2, "0")}</strong><span>In your personal library</span></div>
          <div class="stat hover-glow" onpointermove={moveHoverLight}><small>BOOKMARKED SUBPLACES</small><strong>{games.reduce((n, g) => n + g.subplaces.length, 0).toString().padStart(2, "0")}</strong><span>Quick launch shortcuts</span></div>
          <div class="stat hover-glow" onpointermove={moveHoverLight}><small>BACKGROUND REFRESH</small><strong>OFF</strong><span>Servers load only on request</span></div>
        </div>
        {#if quickLaunchItems.length}
          <div class="section-heading"><div><span class="eyebrow">ONE-CLICK LAUNCH</span><h2>Your favorites</h2></div><button class="link-button" onclick={() => tab = "library"}>Manage library ↗</button></div>
          <div class="quick-launch-grid">
            {#each quickLaunchItems as item (`${item.parentPlaceId}:${item.placeId}`)}
              <div class="quick-game hover-glow" class:subplace={item.isSubplace} onpointermove={moveHoverLight}>
                <button class="quick-game-main" onclick={() => play(item.placeId)} title={"Launch " + item.name} aria-label={"Launch " + item.name}>
                  <span class="game-tile quick-game-icon">{item.name.slice(0, 1).toUpperCase()}
                    {#if isTrustedIconUrl(item.iconUrl)}<img src={item.iconUrl} alt="" loading="lazy" decoding="async" onerror={hideBrokenIcon} />{/if}
                  </span>
                  <span class="quick-game-title">{item.name}</span>
                  {#if item.isSubplace}<span class="quick-game-label" title={item.parentName}>SUBPLACE · {item.parentName}</span>{/if}
                </button>
                <button class="quick-game-server" title={"Browse " + item.name + " servers"} aria-label={"Browse " + item.name + " servers"} onclick={() => { selectGame(item.parentPlaceId); goToServers(item.placeId); }}>◎</button>
              </div>
            {/each}
          </div>
          {#if games.reduce((total, game) => total + 1 + game.subplaces.filter(place => place.pinned).length, 0) > 24}
            <p class="form-hint">Showing the first 24 shortcuts. Your other saved places remain available in My Library.</p>
          {/if}
        {/if}
        <div class="section-heading"><div><span class="eyebrow">QUICK ACCESS</span><h2>Your workspace</h2></div></div>
        <div class="feature-grid">
          <button class="feature hover-glow" onpointermove={moveHoverLight} onclick={() => tab = "library"}><div class="feature-symbol">▦</div><h3>Game library</h3><p>Save experiences and individual subplaces. No account required.</p><span class="feature-link">Explore library ↗</span></button>
          <button class="feature hover-glow" onpointermove={moveHoverLight} onclick={enterServerBrowser}><div class="feature-symbol">◎</div><h3>Public servers</h3><p>Browse server population, temporary regions and Roblox-reported ping.</p><span class="feature-link">Browse servers ↗</span></button>
          <button class="feature hover-glow" onpointermove={moveHoverLight} onclick={() => tab = "settings"}><div class="feature-symbol">⚙</div><h3>Personalize</h3><p>Switch to Lite Mode and keep resource usage predictable.</p><span class="feature-link">Your settings ↗</span></button>
        </div>
      {:else if tab === "library"}
        <div class="page-title"><div><span class="eyebrow">YOUR COLLECTION</span><h1>Game library</h1><p>Bookmark experiences and their individual Roblox places.</p></div><span class="count-badge">{games.length} SAVED</span></div>
        <div class="library-layout">
          <div class="panel">
            <div class="panel-header"><h3>Saved experiences</h3></div>
            <input class="input search" aria-label="Filter saved games" placeholder="⌕  Find a game…" bind:value={filter} />
            <div class="library-sort-controls">
              <label for="library-sort">Sort library</label>
              <select id="library-sort" class="input" value={librarySort} onchange={e => setLibrarySort(e.currentTarget.value as LibrarySort)}>
                <option value="saved">Saved order</option>
                <option value="name">Name A–Z</option>
                <option value="newest">Newest first</option>
                <option value="oldest">Oldest first</option>
              </select>
            </div>
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
              <div class="sub-header"><h3>Saved subplaces</h3><span>{selected.subplaces.length} PLACES · ☆ PINS TO HOME</span></div>
              <div class="subplace-list">
                {#each selected.subplaces as place (place.placeId)}
                  <div class="subplace-row"><div><strong>{place.name}</strong><small>Place ID {place.placeId}</small></div>
                    <div class="row-actions">
                      <button class:pinned={place.pinned === true} aria-pressed={place.pinned === true} title={place.pinned ? "Unpin from home favorites" : "Pin to home favorites"} aria-label={place.pinned ? "Unpin " + place.name : "Pin " + place.name + " to home"} onclick={() => togglePin(place)}>{place.pinned ? "★" : "☆"}</button>
                      <button title="Launch subplace" aria-label={"Launch " + place.name} onclick={() => play(place.placeId)}>▶</button>
                      <button title="View servers" aria-label={"Browse " + place.name + " servers"} onclick={() => goToServers(place.placeId)}>◎</button>
                      <button title="Remove subplace" aria-label={"Remove " + place.name} onclick={() => removeSubplace(place)}>×</button>
                    </div>
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
        <div class="page-title"><div><span class="eyebrow">LIVE DISCOVERY</span><h1>Public servers</h1><p>Public Roblox server data, requested only when you refresh.</p></div><span class="chip">{useAuthenticatedAvatarRequests && connectedAccount ? "SESSION-BACKED REQUESTS" : "PUBLIC REQUESTS"}</span></div>
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
          <div class="rovalra-toolbar">
            <label class="server-checkbox" title="Experimental source; opt in once per Thirst run. No Roblox cookie is shared.">
              <input type="checkbox" checked={rovalraEnabled} onchange={e => setRoValraEnabled(e.currentTarget.checked)} />
              Use RoValra regions (temporary)
            </label>
            <button class="btn secondary small" disabled={!rovalraEnabled || rovalraLoading || !servers.length} onclick={loadRoValraRegions}>
              {rovalraLoading ? "Finding regions…" : "↻ Look up regions"}
            </button>
            <span class="provider-note">Only public place/server IDs are sent to RoValra. No Roblox session cookie. 30-second lookup cooldown.</span>
            <a class="provider-credit" href="https://www.rovalra.com/" target="_blank" rel="noopener noreferrer">Region data by RoValra ↗</a>
          </div>
          <div class="server-disclaimer">Ping is Roblox's <strong>server-reported value, not your personal latency</strong>. Filters and sorting affect only loaded pages, not every live server. Regions are Unknown until you add a local observation or explicitly opt into RoValra's third-party region data. RoValra results are provider-reported, not independently verified by Thirst; local observations expire after two hours. Connected users can see automatic anonymous avatar previews for up to eight loaded servers (unless disabled or in Lite Mode).</div>
          <div class="server-filter-bar">
            <label for="server-sort">Sort
              <select id="server-sort" class="input" bind:value={serverSort}>
                <option value="default">Roblox order</option>
                <option value="lowest-ping">Lowest reported ping</option>
                <option value="most-players">Most players</option>
                <option value="fewest-players">Fewest players</option>
                <option value="most-slots">Most free slots</option>
              </select>
            </label>
            <label for="server-ping">Reported ping
              <select id="server-ping" class="input" bind:value={serverMaxPing}>
                <option value="any">Any</option>
                <option value="50">≤ 50 ms</option>
                <option value="100">≤ 100 ms</option>
                <option value="150">≤ 150 ms</option>
                <option value="200">≤ 200 ms</option>
                <option value="300">≤ 300 ms</option>
              </select>
            </label>
            <label for="server-region">Region · available sources
              <select id="server-region" class="input" bind:value={regionFilter}>
                <option value="any">All regions</option>
                <option value="unknown">Unknown only</option>
                {#each REGION_OPTIONS as option}
                  <option value={option.code}>{option.label}</option>
                {/each}
              </select>
            </label>
            <label for="server-search">Server ID
              <input id="server-search" class="input" placeholder="Search loaded IDs" bind:value={serverSearch} />
            </label>
            <label class="server-checkbox"><input type="checkbox" bind:checked={serverHideFull} /> Free slots only</label>
            
          </div>
          <div class="server-result-note">
            <span>{visibleServers.length} shown / {servers.length} loaded · {servers.filter(server => !!regionReportFor(server.id)).length} local reports · {Object.keys(providerRegions).length} RoValra results</span>
            <button class="link-button" onclick={() => { serverSort = "default"; serverMaxPing = "any"; serverHideFull = false; serverSearch = ""; regionFilter = "any"; }}>Clear filters</button>
            {#if serverLoadError}<span class="server-error">Last request failed: {serverLoadError}</span>{/if}
            {#if rovalraError}<span class="server-error">RoValra: {rovalraError}</span>{/if}
            {#if bulkPreviewError}<span class="preview-error">Avatar preview issue: {bulkPreviewError}</span>{/if}
          </div>
          <div class="table-head"><span>SERVER</span><span>REGION</span><span>PLAYERS</span><span>REPORTED PING</span><span>ACTIONS</span></div>
          {#each visibleServers as server (server.id)}
            <div class="server-row">
              <span class="server-id"><span class="online-dot"></span><button class="server-id-copy" title={"Copy server ID: " + server.id} aria-label={"Copy server ID " + server.id} onclick={() => copyServerId(server.id)}>{server.id.slice(0, 8)}… <span aria-hidden="true">⧉</span></button></span>
              <button
                class="server-region-button"
                class:reported={!!regionReportFor(server.id)}
                class:provider={!regionReportFor(server.id) && !!providerRegions[server.id.toLowerCase()]}
                title={regionReportFor(server.id) ? "Your unverified local report · expires after two hours · click to edit" : providerRegions[server.id.toLowerCase()] ? "RoValra-provided region · not independently verified · click to add your own observation" : "Region unknown · click to add your own report after checking in-game"}
                aria-label={"Region for server " + server.id + ": " + displayServerRegion(server.id) + ". Edit local report."}
                onclick={() => editServerRegion(server.id)}
              >
                {#if regionReportFor(server.id)}<span class="region-source-marker">●</span>{:else if providerRegions[server.id.toLowerCase()]}<span class="region-source-marker provider-marker">◈</span>{/if}
                {displayServerRegion(server.id)}
              </button>
              <span><strong>{server.playing}</strong><span class="muted"> / {server.maxPlayers}</span></span>
              <span>{server.ping === null ? "—" : Math.round(server.ping) + " ms"}</span>
              <div class="server-actions">
                <button class:active={expandedPreviewId === server.id} class="server-preview-button" aria-expanded={expandedPreviewId === server.id} disabled={previewLoadingId !== null && previewLoadingId !== server.id} title="Show up to five anonymous player avatars" onclick={() => toggleServerPreview(server.id)}>Avatars</button>
                <button class="btn small secondary" onclick={() => activePlaceId && play(activePlaceId, server.id)}>Join ↗</button>
              </div>
            </div>
            {#if editingRegionId === server.id}
              <div class="server-region-editor">
                <div class="region-editor-copy">
                  <strong>Report this server's region</strong>
                  <span>Only after you have checked in-game. This is your unverified local label, not a Roblox or RoValra location. It expires in 2 hours.</span>
                </div>
                <label>
                  <span>Region</span>
                  <select class="input" aria-label="Region of this server" bind:value={regionDraft}>
                    <option value="" disabled>Select region</option>
                    {#each REGION_OPTIONS as option}
                      <option value={option.code}>{option.label}</option>
                    {/each}
                  </select>
                </label>
                <div class="region-editor-actions">
                  <button class="btn small primary" disabled={!regionDraft} onclick={() => saveServerRegion(server.id)}>Save locally</button>
                  {#if regionReportFor(server.id)}
                    <button class="btn small secondary" onclick={() => clearServerRegion(server.id)}>Clear</button>
                  {/if}
                  <button class="btn small secondary" onclick={() => editServerRegion(server.id)}>Cancel</button>
                </div>
              </div>
            {/if}
            {#if expandedPreviewId === server.id || (autoAvatarEnabled && connectedAccount && !liteMode && ((avatarPreviews[server.id]?.length ?? 0) > 0 || (bulkPreviewBusy && bulkPreviewRequested.includes(server.id))))}
              <div class="server-avatar-panel" class:auto-loaded={expandedPreviewId !== server.id}>
                <span class="preview-caption">Anonymous public avatar previews · no player names or IDs</span>
                {#if previewLoadingId === server.id || (bulkPreviewBusy && bulkPreviewRequested.includes(server.id))}
                  <span class="preview-status">Requesting up to five anonymous thumbnails from Roblox…</span>
                {:else if avatarErrors[server.id]}
                  <span class="preview-status preview-error">{avatarErrors[server.id]}</span>
                  <button class="btn small secondary" onclick={() => { avatarErrors = { ...avatarErrors, [server.id]: "" }; expandedPreviewId = null; void toggleServerPreview(server.id); }}>Retry</button>
                {:else if avatarPreviews[server.id]?.length}
                  <div class="avatar-image-row">
                    {#each avatarPreviews[server.id] as imageUrl (imageUrl)}
                      <img src={imageUrl} alt="Anonymous Roblox player avatar" loading="lazy" decoding="async" onerror={hideBrokenIcon} />
                    {/each}
                  </div>
                {:else}
                  <span class="preview-status">Roblox did not return any displayable public thumbnails.</span>
                  {#if connectedAccount && !useAuthenticatedAvatarRequests}
                    <button class="link-button" onclick={() => tab = "settings"}>Review experimental session-backed avatar requests in Settings</button>
                  {/if}
                {/if}
              </div>
            {/if}
          {:else}
            <div class="empty-state server-empty"><div class="empty-mark">◎</div>
              <strong>{servers.length ? "No loaded servers match these filters" : "No server data loaded"}</strong>
              <p>{servers.length ? "Try clearing your filters or load another page." : games.length ? "Select a saved place and click Refresh to fetch its public servers." : "Add an experience in your library to browse its servers."}</p>
            </div>
          {/each}
          {#if fetchedAt}
            <div class="server-footer"><span>Last refreshed: {fetchedAt.toLocaleTimeString()} · {servers.length} loaded</span>
              {#if nextCursor}<button class="btn secondary small" disabled={loading} onclick={() => loadServers(true)}>{loading ? "Loading…" : "Load more ↓"}</button>{/if}
            </div>
          {/if}
        </div>
      {:else}
        <div class="page-title"><div><span class="eyebrow">PREFERENCES & PRIVACY</span><h1>Settings</h1><p>Personalize Thirst without compromising performance.</p></div></div>
        <div class="settings-column">
          <section class="panel settings-panel"><div><h3>Lite Mode</h3><p>Disable decorative gradients and transitions to reduce UI overhead.</p></div><label class="switch"><input type="checkbox" checked={liteMode} onchange={e => setLiteMode(e.currentTarget.checked)} /><span class="slider"></span></label></section> 
          <section class="panel settings-panel"><div><h3>Automatic server avatars</h3><p>When connected, request up to five anonymous avatars for each of the first eight servers after Refresh. Guest mode stays manual. Disabled in Lite Mode.</p></div><label class="switch"><input type="checkbox" checked={autoAvatarEnabled} onchange={e => setAutoAvatarEnabled(e.currentTarget.checked)} /><span class="slider"></span></label></section>
          <section class="panel settings-panel"><div><h3>Use saved Roblox session for server avatars</h3><p><strong>Experimental · off by default.</strong> When enabled, native Rust sends your full-access .ROBLOSECURITY cookie only to <code>games.roblox.com</code> for the selected server list and <code>thumbnails.roblox.com</code> for avatar previews. It is never sent to Thirst, RoValra, CDN hosts, or UI scripts. This may prompt macOS Keychain, does not guarantee avatars will appear, and does not bypass Roblox rate limits. Permission resets whenever Thirst restarts. Click Refresh after enabling.</p></div><label class="switch"><input type="checkbox" disabled={!connectedAccount} checked={useAuthenticatedAvatarRequests} onchange={e => setAuthenticatedAvatars(e.currentTarget.checked)} /><span class="slider"></span></label></section>
          <section class="panel settings-panel account-settings">
            <div class="account-info">
              <h3>Roblox account</h3>
              <p>Optional official Roblox login. On macOS Thirst uses a separate native Swift/WebKit sign-in window; on Windows it uses an isolated Tauri webview. Session credentials are handled by native code and stored locally in your OS credential vault, never in the Thirst UI, SQLite, or our server.</p>
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
            <div><strong>Where it will be sent</strong><p>Normally only to Roblox's fixed account-verification endpoint. If you separately enable authenticated server avatars in Settings, native Rust also sends it to fixed games.roblox.com and thumbnails.roblox.com endpoints. No Thirst, RoValra or Discord server receives it.</p></div>
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
          <p class="consent-lead">The separate Roblox sign-in window opens at <strong>https://www.roblox.com/login</strong>. {platform === "macOS" ? "On macOS, the native window automatically closes after detecting a successful session." : "On Windows, complete sign-in in the isolated window."} Then return to Thirst and click <strong>Finish connection</strong>.</p>
          <div class="consent-disclosure">
            <div><strong>No copying cookies</strong><p>Do not paste your .ROBLOSECURITY value anywhere. On macOS, the native helper stores it in a temporary Keychain entry automatically; Thirst verifies and promotes it only when you finish. Windows reads its isolated webview on Finish.</p></div>
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
