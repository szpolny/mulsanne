<script lang="ts">
  let { checked, onchange, label }: { checked: boolean; onchange: (v: boolean) => void; label?: string } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  class={["relative h-5 w-9 shrink-0 rounded-full transition-colors", checked ? "bg-accent" : "bg-surface-3"]}
  onclick={() => onchange(!checked)}
>
  <span
    class="absolute top-0.5 left-0.5 h-4 w-4 rounded-full bg-white shadow transition-transform"
    style:transform={checked ? "translateX(16px)" : "none"}
  ></span>
</button>
