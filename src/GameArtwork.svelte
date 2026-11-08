<script lang="ts">
  import { isTrustedIconUrl } from "./lib/library";

  export let name: string;
  export let src: string | null | undefined = null;
  export let variant: "cover" | "tile" | "hero" = "tile";
  export let eager = false;

  let failed = false;
  let previousUrl: string | null = null;

  $: safeUrl = isTrustedIconUrl(src) ? src : null;
  $: if (safeUrl !== previousUrl) {
    previousUrl = safeUrl;
    failed = false;
  }
  $: fallback = (name.trim().charAt(0) || "?").toUpperCase();
</script>

<!-- A normal-flow square image: no absolute positioning, percent heights,
     clipping transforms or overlaid fallback that might hide the artwork in
     macOS WebKit. The fallback is rendered only if the URL is unusable. -->
<div class="game-art" aria-hidden="true">
  {#if safeUrl && !failed}
    {#key safeUrl}
      <img
        src={safeUrl}
        alt=""
        width="512"
        height="512"
        loading={eager ? "eager" : "lazy"}
        decoding="async"
        draggable="false"
        onerror={() => { failed = true; }}
      />
    {/key}
  {:else}
    <span class="art-fallback" class:tile={variant === "tile"}>{fallback}</span>
  {/if}
</div>

<style>
  .game-art{
    display:block;
    width:100%;
    aspect-ratio:1 / 1;
    overflow:hidden;
    min-width:0;
    background:#24445a;
    border-radius:inherit;
  }
  img{
    display:block;
    width:100%;
    height:auto;
    aspect-ratio:1 / 1;
    object-fit:cover;
    object-position:center;
    max-width:100%;
    border:0;
    border-radius:inherit;
  }
  .art-fallback{
    display:grid;
    place-items:center;
    width:100%;
    aspect-ratio:1 / 1;
    background:linear-gradient(145deg,#315b73,#1d344a);
    color:#e0f7fb;
    font-size:42px;
    font-weight:800;
    line-height:1;
  }
  .tile{font-size:23px}
</style>
