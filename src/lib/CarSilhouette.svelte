<script lang="ts">
  /** A plain prototype outline for cars the game has no render of. */
  let { class: className = "" }: { class?: string } = $props();
</script>

<svg class={className} viewBox="0 0 120 44" fill="none" aria-hidden="true">
  <path
    d="M6 31 L10 23 Q14 19 24 18 L44 16 Q52 8 64 8 L74 8 Q80 8 84 14 L100 18 Q112 20 114 27 L114 31 Z"
    fill="currentColor"
    opacity="0.18"
    stroke="currentColor"
    stroke-width="1.5"
    stroke-linejoin="round"
  />
  <path d="M100 12 L116 12 L116 16" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" opacity="0.6" />
  <circle cx="28" cy="31" r="7" fill="currentColor" opacity="0.5" />
  <circle cx="94" cy="31" r="7" fill="currentColor" opacity="0.5" />
</svg>
