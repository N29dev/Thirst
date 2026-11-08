<script lang="ts">
  import GameArtwork from "./GameArtwork.svelte";
  import type { SavedGame } from "./lib/types";
  export let game: SavedGame;
  $: pinnedCount = game.subplaces.filter(place => place.pinned === true).length;
</script>

<section class="library-hero" aria-label={"Selected experience: " + game.name}>
  <div class="library-hero-art">
    <GameArtwork name={game.name} src={game.iconUrl} variant="hero" eager/>
  </div>
  <div class="library-hero-copy">
    <span class="library-hero-kicker"><span class="library-hero-dot"></span> SAVED EXPERIENCE</span>
    <h2>{game.name}</h2>
    <div class="library-hero-meta">
      <span>Place {game.placeId}</span>
      <span class="library-meta-sep" aria-hidden="true"></span>
      <span>{game.subplaces.length} {game.subplaces.length === 1 ? "subplace" : "subplaces"}</span>
      <span class="library-meta-sep" aria-hidden="true"></span>
      <span>{pinnedCount} pinned</span>
    </div>
  </div>
</section>

<style>
  .library-hero{
    display:flex;align-items:center;gap:20px;min-width:0;
    padding:19px;border:1px solid #365b70;border-radius:17px;
    background:radial-gradient(ellipse at 5% 5%,#43899d27,transparent 54%),
      linear-gradient(105deg,#193748,#142b3c 70%,#122536);
    box-shadow:inset 0 1px 0 #dffaff15;
  }
  .library-hero-art{
    display:block;flex:0 0 118px;width:118px;height:118px;overflow:hidden;
    border:1px solid #79b5cb80;border-radius:19px;background:#173d53;
    box-shadow:0 13px 24px #07152275;
  }
  .library-hero-copy{min-width:0;flex:1}
  .library-hero-kicker{
    display:flex;align-items:center;gap:7px;
    font-size:9px;letter-spacing:1.35px;color:#91d5e1;font-weight:850
  }
  .library-hero-dot{width:6px;height:6px;border-radius:50%;background:#8adbd9;box-shadow:0 0 9px #5dd6ce6e}
  h2{
    color:#ebfaff;font-size:clamp(19px,2.1vw,28px);line-height:1.22;letter-spacing:-.65px;
    margin:10px 0;font-weight:830;overflow-wrap:anywhere
  }
  .library-hero-meta{
    display:flex;align-items:center;gap:10px;flex-wrap:wrap;
    color:#92b6c8;font-size:11px;font-weight:650;line-height:1.55
  }
  .library-meta-sep{width:3px;height:3px;border-radius:50%;background:#547d91;flex:none}
  @media(max-width:680px){
    .library-hero{padding:14px;gap:12px}
    .library-hero-art{flex-basis:88px;width:88px;height:88px;border-radius:13px}
    h2{font-size:19px;margin:7px 0}
    .library-hero-meta{font-size:10px;gap:6px}
  }
  @media(max-width:460px){
    .library-hero{align-items:flex-start}
    .library-hero-art{flex-basis:72px;width:72px;height:72px;border-radius:12px}
    h2{font-size:17px}
    .library-hero-kicker{font-size:8px;letter-spacing:1px}
  }
  :global(.lite) .library-hero,:global(.lite) .library-hero-art{box-shadow:none!important}
</style>
