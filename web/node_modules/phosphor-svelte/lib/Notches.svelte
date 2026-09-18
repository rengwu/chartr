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
    <path d="M216.49,136.49l-80,80a12,12,0,1,1-17-17l80-80a12,12,0,1,1,17,17Zm-16-105a12,12,0,0,0-17,0l-152,152a12,12,0,0,0,17,17l152-152A12,12,0,0,0,200.49,31.51Z"/>
  {:else if weight === "duotone"}
    <path d="M192,40V192H40Z" opacity="0.2"/><path d="M195.06,32.61a8,8,0,0,0-8.72,1.73l-152,152A8,8,0,0,0,40,200H192a8,8,0,0,0,8-8V40A8,8,0,0,0,195.06,32.61ZM184,184H59.31L184,59.31Z"/>
  {:else if weight === "fill"}
    <path d="M200,40V192a8,8,0,0,1-8,8H40a8,8,0,0,1-5.66-13.66l152-152A8,8,0,0,1,200,40Z"/>
  {:else if weight === "light"}
    <path d="M212.24,132.24l-80,80a6,6,0,1,1-8.48-8.48l80-80a6,6,0,1,1,8.48,8.48Zm-16-96.48a6,6,0,0,0-8.48,0l-152,152a6,6,0,1,0,8.48,8.48l152-152A6,6,0,0,0,196.24,35.76Z"/>
  {:else if weight === "regular"}
    <path d="M213.66,133.66l-80,80a8,8,0,0,1-11.32-11.32l80-80a8,8,0,0,1,11.32,11.32Zm-16-99.32a8,8,0,0,0-11.32,0l-152,152a8,8,0,0,0,11.32,11.32l152-152A8,8,0,0,0,197.66,34.34Z"/>
  {:else if weight === "thin"}
    <path d="M210.83,130.83l-80,80a4,4,0,1,1-5.66-5.66l80-80a4,4,0,1,1,5.66,5.66Zm-16-93.66a4,4,0,0,0-5.66,0l-152,152a4,4,0,0,0,5.66,5.66l152-152A4,4,0,0,0,194.83,37.17Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>