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
    <path d="M240.49,136.49l-32,32a12,12,0,0,1-17-17L203,140H53l11.52,11.51a12,12,0,0,1-17,17l-32-32a12,12,0,0,1,0-17l32-32a12,12,0,1,1,17,17L53,116H203l-11.52-11.51a12,12,0,0,1,17-17l32,32A12,12,0,0,1,240.49,136.49Z"/>
  {:else if weight === "duotone"}
    <path d="M232,128l-32,32H56L24,128,56,96H200Z" opacity="0.2"/><path d="M237.66,133.66l-32,32a8,8,0,0,1-11.32-11.32L212.69,136H43.31l18.35,18.34a8,8,0,0,1-11.32,11.32l-32-32a8,8,0,0,1,0-11.32l32-32a8,8,0,0,1,11.32,11.32L43.31,120H212.69l-18.35-18.34a8,8,0,0,1,11.32-11.32l32,32A8,8,0,0,1,237.66,133.66Z"/>
  {:else if weight === "fill"}
    <path d="M237.66,133.66l-32,32A8,8,0,0,1,192,160V136H64v24a8,8,0,0,1-13.66,5.66l-32-32a8,8,0,0,1,0-11.32l32-32A8,8,0,0,1,64,96v24H192V96a8,8,0,0,1,13.66-5.66l32,32A8,8,0,0,1,237.66,133.66Z"/>
  {:else if weight === "light"}
    <path d="M236.24,132.24l-32,32a6,6,0,0,1-8.48-8.48L217.51,134h-179l21.75,21.76a6,6,0,1,1-8.48,8.48l-32-32a6,6,0,0,1,0-8.48l32-32a6,6,0,0,1,8.48,8.48L38.49,122h179l-21.75-21.76a6,6,0,0,1,8.48-8.48l32,32A6,6,0,0,1,236.24,132.24Z"/>
  {:else if weight === "regular"}
    <path d="M237.66,133.66l-32,32a8,8,0,0,1-11.32-11.32L212.69,136H43.31l18.35,18.34a8,8,0,0,1-11.32,11.32l-32-32a8,8,0,0,1,0-11.32l32-32a8,8,0,0,1,11.32,11.32L43.31,120H212.69l-18.35-18.34a8,8,0,0,1,11.32-11.32l32,32A8,8,0,0,1,237.66,133.66Z"/>
  {:else if weight === "thin"}
    <path d="M234.83,130.83l-32,32a4,4,0,0,1-5.66-5.66L222.34,132H33.66l25.17,25.17a4,4,0,0,1-5.66,5.66l-32-32a4,4,0,0,1,0-5.66l32-32a4,4,0,0,1,5.66,5.66L33.66,124H222.34L197.17,98.83a4,4,0,0,1,5.66-5.66l32,32A4,4,0,0,1,234.83,130.83Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>