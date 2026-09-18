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
    <path d="M128,20A108,108,0,1,0,236,128,108.12,108.12,0,0,0,128,20Zm0,192a84,84,0,1,1,84-84A84.09,84.09,0,0,1,128,212Zm-16-84a16,16,0,1,1-16-16A16,16,0,0,1,112,128Zm64,0a16,16,0,1,1-16-16A16,16,0,0,1,176,128Z"/>
  {:else if weight === "duotone"}
    <path d="M224,128a96,96,0,1,1-96-96A96,96,0,0,1,224,128Z" opacity="0.2"/><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216Zm56-88a12,12,0,1,1-12-12A12,12,0,0,1,184,128Zm-44,0a12,12,0,1,1-12-12A12,12,0,0,1,140,128Zm-44,0a12,12,0,1,1-12-12A12,12,0,0,1,96,128Z"/>
  {:else if weight === "fill"}
    <path d="M128,24A104,104,0,1,0,232,128,104.13,104.13,0,0,0,128,24ZM84,140a12,12,0,1,1,12-12A12,12,0,0,1,84,140Zm44,0a12,12,0,1,1,12-12A12,12,0,0,1,128,140Zm44,0a12,12,0,1,1,12-12A12,12,0,0,1,172,140Z"/>
  {:else if weight === "light"}
    <path d="M128,26A102,102,0,1,0,230,128,102.12,102.12,0,0,0,128,26Zm0,192a90,90,0,1,1,90-90A90.1,90.1,0,0,1,128,218Zm10-90a10,10,0,1,1-10-10A10,10,0,0,1,138,128Zm-44,0a10,10,0,1,1-10-10A10,10,0,0,1,94,128Zm88,0a10,10,0,1,1-10-10A10,10,0,0,1,182,128Z"/>
  {:else if weight === "regular"}
    <path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216Zm12-88a12,12,0,1,1-12-12A12,12,0,0,1,140,128Zm44,0a12,12,0,1,1-12-12A12,12,0,0,1,184,128Zm-88,0a12,12,0,1,1-12-12A12,12,0,0,1,96,128Z"/>
  {:else if weight === "thin"}
    <path d="M128,28A100,100,0,1,0,228,128,100.11,100.11,0,0,0,128,28Zm0,192a92,92,0,1,1,92-92A92.1,92.1,0,0,1,128,220Zm8-92a8,8,0,1,1-8-8A8,8,0,0,1,136,128Zm-44,0a8,8,0,1,1-8-8A8,8,0,0,1,92,128Zm88,0a8,8,0,1,1-8-8A8,8,0,0,1,180,128Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>