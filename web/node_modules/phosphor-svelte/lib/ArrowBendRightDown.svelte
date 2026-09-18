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
    <path d="M208.49,184.49l-48,48a12,12,0,0,1-17,0l-48-48a12,12,0,0,1,17-17L140,195V128A84.09,84.09,0,0,0,56,44a12,12,0,0,1,0-24A108.12,108.12,0,0,1,164,128v67l27.51-27.52a12,12,0,0,1,17,17Z"/>
  {:else if weight === "duotone"}
    <path d="M200,176l-48,48-48-48Z" opacity="0.2"/><path d="M207.39,172.94A8,8,0,0,0,200,168H160V128A104.11,104.11,0,0,0,56,24a8,8,0,0,0,0,16,88.1,88.1,0,0,1,88,88v40H104a8,8,0,0,0-5.66,13.66l48,48a8,8,0,0,0,11.32,0l48-48A8,8,0,0,0,207.39,172.94ZM152,212.69,123.31,184h57.38Z"/>
  {:else if weight === "fill"}
    <path d="M205.66,181.66l-48,48a8,8,0,0,1-11.32,0l-48-48A8,8,0,0,1,104,168h40V128A88.1,88.1,0,0,0,56,40a8,8,0,0,1,0-16A104.11,104.11,0,0,1,160,128v40h40a8,8,0,0,1,5.66,13.66Z"/>
  {:else if weight === "light"}
    <path d="M204.24,180.24l-48,48a6,6,0,0,1-8.48,0l-48-48a6,6,0,0,1,8.48-8.48L146,209.51V128A90.1,90.1,0,0,0,56,38a6,6,0,0,1,0-12A102.12,102.12,0,0,1,158,128v81.51l37.76-37.75a6,6,0,0,1,8.48,8.48Z"/>
  {:else if weight === "regular"}
    <path d="M205.66,181.66l-48,48a8,8,0,0,1-11.32,0l-48-48a8,8,0,0,1,11.32-11.32L144,204.69V128A88.1,88.1,0,0,0,56,40a8,8,0,0,1,0-16A104.11,104.11,0,0,1,160,128v76.69l34.34-34.35a8,8,0,0,1,11.32,11.32Z"/>
  {:else if weight === "thin"}
    <path d="M202.83,178.83l-48,48a4,4,0,0,1-5.66,0l-48-48a4,4,0,0,1,5.66-5.66L148,214.34V128A92.1,92.1,0,0,0,56,36a4,4,0,0,1,0-8A100.11,100.11,0,0,1,156,128v86.34l41.17-41.17a4,4,0,0,1,5.66,5.66Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>