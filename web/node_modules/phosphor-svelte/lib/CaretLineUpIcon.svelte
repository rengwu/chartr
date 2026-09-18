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
    <path d="M216.49,183.51a12,12,0,0,1-17,17L128,129,56.49,200.49a12,12,0,0,1-17-17l80-80a12,12,0,0,1,17,0ZM48,84H208a12,12,0,0,0,0-24H48a12,12,0,0,0,0,24Z"/>
  {:else if weight === "duotone"}
    <path d="M208,192H48l80-80Z" opacity="0.2"/><path d="M133.66,106.34a8,8,0,0,0-11.32,0l-80,80A8,8,0,0,0,48,200H208a8,8,0,0,0,5.66-13.66ZM67.31,184,128,123.31,188.69,184ZM40,72a8,8,0,0,1,8-8H208a8,8,0,0,1,0,16H48A8,8,0,0,1,40,72Z"/>
  {:else if weight === "fill"}
    <path d="M213.66,186.34A8,8,0,0,1,208,200H48a8,8,0,0,1-5.66-13.66l80-80a8,8,0,0,1,11.32,0ZM48,80H208a8,8,0,0,0,0-16H48a8,8,0,0,0,0,16Z"/>
  {:else if weight === "light"}
    <path d="M212.24,196.24a6,6,0,0,1-8.48,0L128,120.49,52.24,196.24a6,6,0,0,1-8.48-8.48l80-80a6,6,0,0,1,8.48,0l80,80A6,6,0,0,1,212.24,196.24ZM48,78H208a6,6,0,0,0,0-12H48a6,6,0,0,0,0,12Z"/>
  {:else if weight === "regular"}
    <path d="M213.66,197.66a8,8,0,0,1-11.32,0L128,123.31,53.66,197.66a8,8,0,0,1-11.32-11.32l80-80a8,8,0,0,1,11.32,0l80,80A8,8,0,0,1,213.66,197.66ZM48,80H208a8,8,0,0,0,0-16H48a8,8,0,0,0,0,16Z"/>
  {:else if weight === "thin"}
    <path d="M210.83,194.83a4,4,0,0,1-5.66,0L128,117.66,50.83,194.83a4,4,0,0,1-5.66-5.66l80-80a4,4,0,0,1,5.66,0l80,80A4,4,0,0,1,210.83,194.83ZM48,76H208a4,4,0,0,0,0-8H48a4,4,0,0,0,0,8Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>