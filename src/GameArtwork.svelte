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

<!-- Explicit square grid sizing avoids a WebKit layout issue with nested
     height:100% and absolutely positioned images inside aspect-ratio boxes.
     The placeholder stays underneath the image, not in a different layer. -->
<div class="game-art" class:tile={variant === "tile"} aria-hidden="true">
  <span class="art-fallback">{fallback}</span>
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
  {/if}
</div>

<style>
  .game-art {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(0, 1fr);
    width: 100%;
    aspect-ratio: 1 / 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    border-radius: inherit;
    background: #213b51;
  }
  .art-fallback, img {
    grid-column: 1;
    grid-row: 1;
    display: block;
    width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
  }
  .art-fallback {
    display: grid;
    place-items: center;
    background: linear-gradient(145deg, #315b73, #1d344a);
    color: #e0f7fb;
    font-size: 42px;
    font-weight: 800;
    line-height: 1;
  }
  .tile .art-fallback { font-size: 23px; }
  img {
    z-index: 1;
    object-fit: cover;
    object-position: center;
    max-width: 100%;
    max-height: 100%;
    border: 0;
    border-radius: inherit;
  }
</style>
