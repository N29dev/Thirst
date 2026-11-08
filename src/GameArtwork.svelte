<script lang="ts">
  import { isTrustedIconUrl } from "./lib/library";

  export let name: string;
  export let src: string | null | undefined = null;
  export let variant: "cover" | "tile" | "hero" = "tile";
  export let eager = false;

  let loaded = false;
  let failed = false;
  let previousUrl: string | null = null;

  // Reuse the existing public Roblox CDN allowlist. Never construct an image
  // URL or load artwork from user-supplied domains.
  $: safeUrl = isTrustedIconUrl(src) ? src : null;
  $: if (safeUrl !== previousUrl) {
    previousUrl = safeUrl;
    loaded = false;
    failed = false;
  }
  $: fallback = (name.trim().charAt(0) || "?").toUpperCase();
</script>

<div class="game-art" class:ready={loaded} class:missing={!safeUrl || failed} class:cover={variant === "cover"} class:tile={variant === "tile"} class:hero={variant === "hero"} aria-hidden="true">
  <div class="art-fallback"><span>{fallback}</span></div>
  {#if safeUrl && !failed}
    <img src={safeUrl} alt="" loading={eager ? "eager" : "lazy"} decoding="async"
      onload={() => { loaded = true; }} onerror={() => { failed = true; loaded = false; }} />
  {/if}
  <span class="art-sheen"></span>
</div>

<style>
  .game-art{position:relative;display:block;width:100%;height:100%;min-width:0;overflow:hidden;isolation:isolate;background:#1d3951}
  .art-fallback{position:absolute;inset:0;display:grid;place-items:center;
    background:radial-gradient(circle at 32% 24%,#406a86 0%,#1f405d 46%,#172c45 100%);
    color:#b9ecf8}
  .art-fallback::before{content:"";position:absolute;inset:-35%;
    border:1px solid #b8eaf521;border-radius:38%;transform:rotate(-24deg)}
  .art-fallback span{font-size:clamp(28px,5vw,72px);font-weight:850;line-height:1;letter-spacing:-4px;opacity:.42}
  .tile .art-fallback span{font-size:27px;letter-spacing:-2px;opacity:.75}
  .hero .art-fallback span{font-size:78px}
  img{position:absolute;inset:0;width:100%;height:100%;object-fit:cover;
    opacity:0;transform:scale(1.055);transition:opacity .3s ease,transform .45s cubic-bezier(.2,.75,.2,1)}
  .ready img{opacity:1;transform:scale(1)}
  .art-sheen{position:absolute;inset:0;pointer-events:none;background:linear-gradient(160deg,#fff8 0%,transparent 32%,#050d1a2b 100%);opacity:.14}
  .cover .art-sheen{opacity:.30}
  .hero .art-sheen{opacity:.2}
  @media(prefers-reduced-motion:reduce){img{transition:none;transform:none}}
  :global(.lite) img{transition:none!important;transform:none!important}
</style>
