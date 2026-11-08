<script lang="ts">
  import GameArtwork from "./GameArtwork.svelte";
  import UiIcon from "./UiIcon.svelte";
  import type { SavedGame, SavedPlace } from "./lib/types";

  export let game: SavedGame;
  export let pinned: SavedPlace[];
  export let onLaunch: (placeId: number) => void | Promise<unknown>;
  export let onBrowse: (gameId: number, placeId: number) => void;
</script>

<article class="favorite-experience">
  <div class="favorite-head">
    <div class="favorite-cover">
      <GameArtwork name={game.name} src={game.iconUrl} variant="hero" eager/>
    </div>
    <div class="favorite-description">
      <div class="favorite-kicker">
        <span class="favorite-light"></span>
        SAVED EXPERIENCE
      </div>
      <h3 title={game.name}>{game.name}</h3>
      <p class="favorite-detail">Your main Roblox experience</p>
      <div class="favorite-primary-actions">
        <button class="favorite-play" aria-label={"Play " + game.name}
          onclick={() => void onLaunch(game.placeId)}>
          <UiIcon name="play" size={16}/>
          <span>Play now</span>
        </button>
        <button class="favorite-browse" aria-label={"Browse servers for " + game.name}
          onclick={() => onBrowse(game.placeId,game.placeId)}>
          <UiIcon name="server" size={16}/>
          <span>Servers</span>
        </button>
      </div>
    </div>
  </div>
  {#if pinned.length}
    <div class="favorite-pinned-heading">
      <span>PINNED PLACES</span>
      <span class="favorite-pinned-count">{pinned.length}</span>
    </div>
    <div class="favorite-place-list">
      {#each pinned as place (place.placeId)}
        <div class="favorite-place">
          <span class="favorite-place-mark"><UiIcon name="server" size={17}/></span>
          <span class="favorite-place-label"><strong title={place.name}>{place.name}</strong><small>Quick launch · Place {place.placeId}</small></span>
          <div class="favorite-place-actions">
            <button class="favorite-place-play" title={"Play " + place.name}
              aria-label={"Play " + place.name} onclick={() => void onLaunch(place.placeId)}>
              <UiIcon name="play" size={15}/><span>Play</span>
            </button>
            <button class="favorite-place-server" title={"Browse servers for " + place.name}
              aria-label={"Browse servers for " + place.name} onclick={() => onBrowse(game.placeId,place.placeId)}>
              <UiIcon name="server" size={16}/><span class="sr-only">Servers</span>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</article>

<style>
  .favorite-experience{
    overflow:hidden;min-width:0;border-radius:21px;
    border:1px solid #3c6176;
    background:linear-gradient(130deg,#162e40 0%,#122638 54%,#111f30 100%);
    box-shadow:inset 0 1px 0 #d8fbff17,0 18px 45px #020b1450;
  }
  .favorite-head{
    display:grid;grid-template-columns:150px minmax(0,1fr);
    gap:23px;align-items:center;padding:23px;
    background:radial-gradient(ellipse at 0% 0%,#4b9fb526,transparent 61%);
  }
  .favorite-cover{
    width:150px;aspect-ratio:1/1;min-width:0;overflow:hidden;border-radius:20px;
    border:1px solid #9edff15b;background:#1c4054;
    box-shadow:0 16px 30px #07131f75,0 0 0 5px #9cd7e708;
    transform:translateZ(0);
  }
  .favorite-description{min-width:0;display:flex;flex-direction:column;align-items:flex-start;gap:10px}
  .favorite-kicker{
    display:flex;align-items:center;gap:8px;
    color:#87cfdc;font-size:9px;font-weight:850;letter-spacing:1.5px
  }
  .favorite-light{width:7px;height:7px;border-radius:50%;background:#76e1d2;box-shadow:0 0 11px #62ddc779}
  h3{margin:0;max-width:100%;font-size:clamp(17px,1.65vw,24px);line-height:1.25;
    font-weight:830;letter-spacing:-.5px;color:#eef9ff;overflow-wrap:anywhere}
  .favorite-detail{margin:0;color:#8baabd;font-size:11px}
  .favorite-primary-actions{display:flex;align-items:center;gap:9px;margin-top:8px;flex-wrap:wrap}
  .favorite-primary-actions button,.favorite-place-actions button{
    display:inline-flex;justify-content:center;align-items:center;gap:8px;
    min-height:37px;padding:9px 14px;border-radius:10px;
    font-size:11px;font-weight:800;line-height:1.3;cursor:pointer;
    transition:transform .15s ease,background .15s ease,border-color .15s ease;
  }
  .favorite-play{
    border:1px solid #a8f4f8;background:linear-gradient(125deg,#b2f3f8,#69d9e7);
    color:#092331;box-shadow:0 6px 22px #4bcfdd21
  }
  .favorite-play:hover{background:#ddfbfd;transform:translateY(-1px)}
  .favorite-browse{
    border:1px solid #41677c;background:#19394a;color:#cce9f4
  }
  .favorite-browse:hover{background:#28566c;border-color:#77b4c8}
  .favorite-pinned-heading{
    padding:17px 23px 10px;display:flex;align-items:center;gap:10px;
    border-top:1px solid #35566a;background:#0f2131c4;
    color:#8bb9ce;font-size:9px;font-weight:850;letter-spacing:1.5px
  }
  .favorite-pinned-count{padding:3px 7px;background:#25475b;border:1px solid #477c92;
    color:#b6e5f1;border-radius:6px;font-size:10px}
  .favorite-place-list{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));
    gap:10px;padding:9px 19px 19px;background:#0f2131c4}
  .favorite-place{
    display:flex;align-items:center;gap:11px;min-width:0;
    padding:12px 13px;border:1px solid #315163;border-radius:13px;
    background:linear-gradient(135deg,#1b3444,#162b3b);
    transition:border-color .16s ease,background .16s ease
  }
  .favorite-place:hover{border-color:#6aa8be;background:#234456}
  .favorite-place-mark{
    display:grid;place-items:center;flex:0 0 35px;width:35px;height:35px;border-radius:10px;
    border:1px solid #50839a;background:#24495c;color:#9bdeed
  }
  .favorite-place-label{display:flex;flex-direction:column;gap:5px;min-width:0;flex:1}
  .favorite-place-label strong{font-size:11px;font-weight:800;color:#ebf6fe;
    white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
  .favorite-place-label small{font-size:9px;color:#89aabd;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
  .favorite-place-actions{display:flex;align-items:center;gap:6px;flex:0 0 auto}
  .favorite-place-actions button{min-height:31px;padding:7px 9px;font-size:10px;border:1px solid #43687c}
  .favorite-place-play{background:#244b5f;color:#c5f2f8}
  .favorite-place-server{background:#132c3c;color:#a9d1e2}
  .favorite-place-actions button:hover{border-color:#8ad5e5;background:#32637a}
  button:focus-visible{outline:2px solid #a7f2fd;outline-offset:3px}
  .sr-only{position:absolute;width:1px;height:1px;padding:0;margin:-1px;
    overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
  @media(min-width:1360px){
    .favorite-head{grid-template-columns:165px minmax(0,1fr)}
    .favorite-cover{width:165px}
  }
  @media(max-width:900px){
    .favorite-head{grid-template-columns:125px minmax(0,1fr);gap:15px;padding:18px}
    .favorite-cover{width:125px;border-radius:17px}
    .favorite-place-list{grid-template-columns:1fr;padding:9px 14px 14px}
    .favorite-pinned-heading{padding:14px 18px 9px}
  }
  @media(max-width:520px){
    .favorite-head{grid-template-columns:92px minmax(0,1fr);padding:14px;gap:13px;align-items:start}
    .favorite-cover{width:92px;border-radius:13px}
    .favorite-description{gap:7px}
    h3{font-size:15px}
    .favorite-detail{font-size:10px}
    .favorite-kicker{font-size:8px;letter-spacing:1px}
    .favorite-primary-actions{grid-column:1/-1;margin-top:4px;gap:6px}
    .favorite-primary-actions button{padding:8px 10px;min-height:32px;font-size:10px}
    .favorite-place{padding:10px;gap:8px}
    .favorite-place-mark{width:29px;height:29px;flex-basis:29px}
  }
  @media(prefers-reduced-motion:reduce){
    .favorite-experience *, .favorite-experience *::before, .favorite-experience *::after{transition:none!important;animation:none!important}
    .favorite-play:hover{transform:none}
  }
  :global(.lite) .favorite-experience,:global(.lite) .favorite-cover{box-shadow:none}
  :global(.lite) .favorite-experience *{transition:none!important;animation:none!important}
</style>
