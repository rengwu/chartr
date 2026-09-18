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
    <path d="M216.49,191.51a12,12,0,0,1-17,17L128,137,56.49,208.49a12,12,0,0,1-17-17l80-80a12,12,0,0,1,17,0Zm-160-63L128,57l71.51,71.52a12,12,0,0,0,17-17l-80-80a12,12,0,0,0-17,0l-80,80a12,12,0,0,0,17,17Z"/>
  {:else if weight === "duotone"}
    <path d="M208,200H48l80-80Z" opacity="0.2"/><path d="M133.66,114.34a8,8,0,0,0-11.32,0l-80,80A8,8,0,0,0,48,208H208a8,8,0,0,0,5.66-13.66ZM67.31,192,128,131.31,188.69,192Zm-25-66.34a8,8,0,0,1,0-11.32l80-80a8,8,0,0,1,11.32,0l80,80a8,8,0,0,1-11.32,11.32L128,51.31,53.66,125.66A8,8,0,0,1,42.34,125.66Z"/>
  {:else if weight === "fill"}
    <path d="M213.66,194.34A8,8,0,0,1,208,208H48a8,8,0,0,1-5.66-13.66L108.69,128H48a8,8,0,0,1-5.66-13.66l80-80a8,8,0,0,1,11.32,0l80,80A8,8,0,0,1,208,128H147.31Z"/>
  {:else if weight === "light"}
    <path d="M212.24,195.76a6,6,0,1,1-8.48,8.48L128,128.49,52.24,204.24a6,6,0,0,1-8.48-8.48l80-80a6,6,0,0,1,8.48,0Zm-160-71.52L128,48.49l75.76,75.75a6,6,0,0,0,8.48-8.48l-80-80a6,6,0,0,0-8.48,0l-80,80a6,6,0,0,0,8.48,8.48Z"/>
  {:else if weight === "regular"}
    <path d="M213.66,194.34a8,8,0,0,1-11.32,11.32L128,131.31,53.66,205.66a8,8,0,0,1-11.32-11.32l80-80a8,8,0,0,1,11.32,0Zm-160-68.68L128,51.31l74.34,74.35a8,8,0,0,0,11.32-11.32l-80-80a8,8,0,0,0-11.32,0l-80,80a8,8,0,0,0,11.32,11.32Z"/>
  {:else if weight === "thin"}
    <path d="M210.83,197.17a4,4,0,0,1-5.66,5.66L128,125.66,50.83,202.83a4,4,0,0,1-5.66-5.66l80-80a4,4,0,0,1,5.66,0Zm-160-74.34L128,45.66l77.17,77.17a4,4,0,1,0,5.66-5.66l-80-80a4,4,0,0,0-5.66,0l-80,80a4,4,0,0,0,5.66,5.66Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>