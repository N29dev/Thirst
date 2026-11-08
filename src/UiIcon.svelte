<script lang="ts">
  type IconName = "home" | "library" | "server" | "flags" | "settings" | "play" | "arrow-up-right" | "sparkles" | "refresh" | "search";
  export let name: IconName;
  export let size = 20;
  export let stroke = 1.8;
</script>

<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24"
  fill="none" stroke="currentColor" stroke-width={stroke} stroke-linecap="round"
  stroke-linejoin="round" aria-hidden="true" focusable="false" class="ui-icon">
  {#if name === "home"}
    <path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1Z"/>
    <path d="M9 21v-8h6v8"/>
  {:else if name === "library"}
    <rect x="3" y="3" width="7" height="7" rx="1.6"/>
    <rect x="14" y="3" width="7" height="7" rx="1.6"/>
    <rect x="3" y="14" width="7" height="7" rx="1.6"/>
    <rect x="14" y="14" width="7" height="7" rx="1.6"/>
  {:else if name === "server"}
    <rect x="3" y="3" width="18" height="7" rx="2"/>
    <rect x="3" y="14" width="18" height="7" rx="2"/>
    <path d="M7 6.5h.01M7 17.5h.01M11 6.5h6M11 17.5h6"/>
  {:else if name === "flags"}
    <path d="M5 21V5m0-1c4-4 10 4 14 0v11c-4 4-10-4-14 0"/>
  {:else if name === "settings"}
    <path d="M12 3 14 5l2.6-.4.7 2.7 2.5 1.3-1.1 2.5 1.1 2.5-2.5 1.3-.7 2.7L14 17l-2 2-2-2-2.6.4-.7-2.7-2.5-1.3 1.1-2.5-1.1-2.5 2.5-1.3.7-2.7L10 5z" transform="translate(0 -1)"/>
    <circle cx="12" cy="11" r="3"/>
  {:else if name === "play"}
    <path d="m8 5 11 7-11 7z"/>
  {:else if name === "arrow-up-right"}
    <path d="M7 17 17 7M7 7h10v10"/>
  {:else if name === "sparkles"}
    <path d="m12 2 1.8 6.2L20 10l-6.2 1.8L12 18l-1.8-6.2L4 10l6.2-1.8zM19 18l.6 1.4L21 20l-1.4.6L19 22l-.6-1.4L17 20l1.4-.6z"/>
  {:else if name === "refresh"}
    <path d="M20 11a8 8 0 0 0-14-5L3 9m0-6v6h6M4 13a8 8 0 0 0 14 5l3-3m0 6v-6h-6"/>
  {:else if name === "search"}
    <circle cx="10.8" cy="10.8" r="7.3"/><path d="m16 16 5 5"/>
  {/if}
</svg>

<style>
  .ui-icon{display:block;flex-shrink:0}
</style>
