<!-- GENERATED FILE -->
<script lang="ts">
  import type { IconComponentProps } from "./shared.d.ts";
  import { getIconContext } from "./context";

  const ctx = getIconContext();

  let { children, ...props }: IconComponentProps = $props();

  let weight = $derived(props.weight ?? ctx.weight ?? "regular");
  let color = $derived(props.color ?? ctx.color ?? "currentColor");
  let size = $derived(props.size ?? ctx.size ?? "1em");
  let mirrored = $derived(props.mirrored ?? ctx.mirrored ?? false);

  function svgAttr(obj: IconComponentProps) {
    let { weight, color, size, mirrored, ...attrs } = obj;
    return attrs;
  }
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  role="img"
  width={size}
  height={size}
  fill={color}
  transform={mirrored ? "scale(-1, 1)" : undefined}
  viewBox="0 0 256 256"
  {...svgAttr(ctx)}
  {...svgAttr(props)}
>
  {#if children}
    {@render children()}
  {/if}
  <rect width="256" height="256" fill="none" />
  {#if weight === "bold"}
    <path d="M220,200a12,12,0,0,1-12,12H56a12,12,0,0,1,0-24H208A12,12,0,0,1,220,200Zm-68-52H56a12,12,0,0,0,0,24h96a68,68,0,0,0,0-136H56a12,12,0,0,0,0,24h96a44,44,0,0,1,0,88Z"/>
  {:else if weight === "duotone"}
    <path d="M208,104a56,56,0,0,1-56,56H56V48h96A56,56,0,0,1,208,104Z" opacity="0.2"/><path d="M216,200a8,8,0,0,1-8,8H56a8,8,0,0,1,0-16H208A8,8,0,0,1,216,200Zm-64-48H56a8,8,0,0,0,0,16h96a64,64,0,0,0,0-128H56a8,8,0,0,0,0,16h96a48,48,0,0,1,0,96Z"/>
  {:else if weight === "fill"}
    <path d="M208,32H48A16,16,0,0,0,32,48V208a16,16,0,0,0,16,16H208a16,16,0,0,0,16-16V48A16,16,0,0,0,208,32ZM176,184H80a8,8,0,0,1,0-16h96a8,8,0,0,1,0,16Zm-32-32H80a8,8,0,0,1,0-16h64a24,24,0,0,0,0-48H80a8,8,0,0,1,0-16h64a40,40,0,0,1,0,80Z"/>
  {:else if weight === "light"}
    <path d="M214,200a6,6,0,0,1-6,6H56a6,6,0,0,1,0-12H208A6,6,0,0,1,214,200Zm-62-46H56a6,6,0,0,0,0,12h96a62,62,0,0,0,0-124H56a6,6,0,0,0,0,12h96a50,50,0,0,1,0,100Z"/>
  {:else if weight === "regular"}
    <path d="M216,200a8,8,0,0,1-8,8H56a8,8,0,0,1,0-16H208A8,8,0,0,1,216,200Zm-64-48H56a8,8,0,0,0,0,16h96a64,64,0,0,0,0-128H56a8,8,0,0,0,0,16h96a48,48,0,0,1,0,96Z"/>
  {:else if weight === "thin"}
    <path d="M212,200a4,4,0,0,1-4,4H56a4,4,0,0,1,0-8H208A4,4,0,0,1,212,200Zm-60-44H56a4,4,0,0,0,0,8h96a60,60,0,0,0,0-120H56a4,4,0,0,0,0,8h96a52,52,0,0,1,0,104Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>