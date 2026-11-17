<script lang="ts" generics="T extends string | number">
  import { tick } from "svelte";

  type SelectOption<V> = { value: V; label: string };

  /** Dropdown styled like the rest of the app, replacing the native `<select>`. */
  let {
    value = $bindable(),
    options,
    onchange,
    onopen,
    label,
    title,
    placeholder = "Select…",
    class: klass = "",
    size = "md",
  }: {
    value: T | null;
    options: SelectOption<T>[];
    onchange?: (value: T) => void;
    onopen?: () => void;
    label?: string;
    title?: string;
    placeholder?: string;
    class?: string;
    size?: "sm" | "md";
  } = $props();

  const id = `select-${Math.random().toString(36).slice(2, 9)}`;
  let open = $state(false);
  let active = $state(-1);
  let button = $state<HTMLButtonElement>();
  let list = $state<HTMLUListElement>();
  let pos = $state({ top: 0, left: 0, width: 0, maxHeight: 288, up: false });

  const current = $derived(options.find((o) => o.value === value));

  function place() {
    if (!button) return;
    const r = button.getBoundingClientRect();
    const below = window.innerHeight - r.bottom - 12;
    const above = r.top - 12;
    const up = below < 200 && above > below;
    pos = {
      top: up ? r.top - 4 : r.bottom + 4,
      left: r.left,
      width: r.width,
      maxHeight: Math.min(288, up ? above : below),
      up,
    };
  }

  async function show() {
    if (open) return;
    onopen?.();
    place();
    active = Math.max(0, options.findIndex((o) => o.value === value));
    open = true;
    await tick();
    scrollActive();
  }

  function hide() {
    open = false;
  }

  function choose(i: number) {
    const o = options[i];
    if (!o) return;
    hide();
    button?.focus();
    if (o.value === value) return;
    value = o.value;
    onchange?.(o.value);
  }

  function scrollActive() {
    list?.querySelector<HTMLElement>(`[data-index="${active}"]`)?.scrollIntoView({ block: "nearest" });
  }

  function move(to: number) {
    active = Math.max(0, Math.min(options.length - 1, to));
    tick().then(scrollActive);
  }

  function onKey(e: KeyboardEvent) {
    if (!open) {
      if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
        e.preventDefault();
        show();
      }
      return;
    }
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        move(active + 1);
        break;
      case "ArrowUp":
        e.preventDefault();
        move(active - 1);
        break;
      case "Home":
        e.preventDefault();
        move(0);
        break;
      case "End":
        e.preventDefault();
        move(options.length - 1);
        break;
      case "Enter":
      case " ":
        e.preventDefault();
        choose(active);
        break;
      case "Escape":
      case "Tab":
        hide();
        break;
      default:
        // Type-ahead: jump to the next option starting with the typed letter.
        if (e.key.length === 1) {
          const k = e.key.toLowerCase();
          const n = options.length;
          for (let i = 1; i <= n; i++) {
            const j = (active + i) % n;
            if (options[j].label.toLowerCase().startsWith(k)) {
              move(j);
              break;
            }
          }
        }
    }
  }

  function onWindowPointer(e: PointerEvent) {
    if (!open) return;
    const t = e.target as Node;
    if (!button?.contains(t) && !list?.contains(t)) hide();
  }
</script>

<svelte:window
  onpointerdown={onWindowPointer}
  onresize={() => open && hide()}
  onscrollcapture={(e) => open && !list?.contains(e.target as Node) && hide()}
/>

<button
  bind:this={button}
  type="button"
  role="combobox"
  aria-haspopup="listbox"
  aria-expanded={open}
  aria-controls={id}
  aria-activedescendant={open && active >= 0 ? `${id}-${active}` : undefined}
  aria-label={label}
  {title}
  class={[
    "field flex items-center justify-between gap-2 text-left",
    size === "sm" && "py-1.5! text-xs!",
    open && "border-soft/40!",
    klass,
  ]}
  onclick={() => (open ? hide() : show())}
  onkeydown={onKey}
>
  <span class={["min-w-0 truncate", !current && "text-muted"]}>{current?.label ?? placeholder}</span>
  <svg
    viewBox="0 0 12 12"
    class={["h-3 w-3 shrink-0 text-muted transition-transform", open && "rotate-180"]}
    aria-hidden="true"
  >
    <path d="M2.5 4.5 6 8l3.5-3.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
  </svg>
</button>

{#if open}
  <ul
    bind:this={list}
    {id}
    role="listbox"
    aria-label={label}
    class="fixed z-50 overflow-y-auto rounded-lg border border-line bg-surface-2 p-1 shadow-[0_16px_40px_rgb(0_0_0/0.55)]"
    style:left="{pos.left}px"
    style:top={pos.up ? undefined : `${pos.top}px`}
    style:bottom={pos.up ? `${window.innerHeight - pos.top}px` : undefined}
    style:min-width="{pos.width}px"
    style:max-width="min(32rem, calc(100vw - {pos.left + 12}px))"
    style:max-height="{pos.maxHeight}px"
  >
    {#each options as o, i (o.value)}
      {@const selected = o.value === value}
      <li
        id="{id}-{i}"
        data-index={i}
        role="option"
        aria-selected={selected}
        class={[
          "flex cursor-default items-center gap-2 rounded-md px-2.5 py-1.5 text-sm",
          i === active ? "bg-surface-3 text-fg" : selected ? "text-fg" : "text-soft",
        ]}
        onpointermove={() => (active = i)}
        onclick={() => choose(i)}
        onkeydown={() => {}}
      >
        <span class="min-w-0 flex-1 truncate">{o.label}</span>
        {#if selected}
          <svg viewBox="0 0 12 12" class="h-3 w-3 shrink-0 text-accent-hi" aria-hidden="true">
            <path d="m2.5 6.2 2.3 2.3 4.7-5" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        {/if}
      </li>
    {/each}
  </ul>
{/if}
