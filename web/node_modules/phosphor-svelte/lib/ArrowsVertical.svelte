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
    <path d="M168.49,191.51a12,12,0,0,1,0,17l-32,32a12,12,0,0,1-17,0l-32-32a12,12,0,0,1,17-17L116,203V53L104.49,64.49a12,12,0,1,1-17-17l32-32a12,12,0,0,1,17,0l32,32a12,12,0,0,1-17,17L140,53V203l11.51-11.52A12,12,0,0,1,168.49,191.51Z"/>
  {:else if weight === "duotone"}
    <path d="M160,56V200l-32,32L96,200V56l32-32Z" opacity="0.2"/><path d="M165.66,194.34a8,8,0,0,1,0,11.32l-32,32a8,8,0,0,1-11.32,0l-32-32a8,8,0,0,1,11.32-11.32L120,212.69V43.31L101.66,61.66A8,8,0,0,1,90.34,50.34l32-32a8,8,0,0,1,11.32,0l32,32a8,8,0,0,1-11.32,11.32L136,43.31V212.69l18.34-18.35A8,8,0,0,1,165.66,194.34Z"/>
  {:else if weight === "fill"}
    <path d="M167.39,196.94a8,8,0,0,1-1.73,8.72l-32,32a8,8,0,0,1-11.32,0l-32-32A8,8,0,0,1,96,192h24V64H96a8,8,0,0,1-5.66-13.66l32-32a8,8,0,0,1,11.32,0l32,32A8,8,0,0,1,160,64H136V192h24A8,8,0,0,1,167.39,196.94Z"/>
  {:else if weight === "light"}
    <path d="M164.24,195.76a6,6,0,0,1,0,8.48l-32,32a6,6,0,0,1-8.48,0l-32-32a6,6,0,0,1,8.48-8.48L122,217.51v-179L100.24,60.24a6,6,0,0,1-8.48-8.48l32-32a6,6,0,0,1,8.48,0l32,32a6,6,0,1,1-8.48,8.48L134,38.49v179l21.76-21.75A6,6,0,0,1,164.24,195.76Z"/>
  {:else if weight === "regular"}
    <path d="M165.66,194.34a8,8,0,0,1,0,11.32l-32,32a8,8,0,0,1-11.32,0l-32-32a8,8,0,0,1,11.32-11.32L120,212.69V43.31L101.66,61.66A8,8,0,0,1,90.34,50.34l32-32a8,8,0,0,1,11.32,0l32,32a8,8,0,0,1-11.32,11.32L136,43.31V212.69l18.34-18.35A8,8,0,0,1,165.66,194.34Z"/>
  {:else if weight === "thin"}
    <path d="M162.83,197.17a4,4,0,0,1,0,5.66l-32,32a4,4,0,0,1-5.66,0l-32-32a4,4,0,0,1,5.66-5.66L124,222.34V33.66L98.83,58.83a4,4,0,0,1-5.66-5.66l32-32a4,4,0,0,1,5.66,0l32,32a4,4,0,0,1-5.66,5.66L132,33.66V222.34l25.17-25.17A4,4,0,0,1,162.83,197.17Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>