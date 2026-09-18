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
    <path d="M176,76a48,48,0,1,0-48,48A48.05,48.05,0,0,0,176,76Zm-48,24a24,24,0,1,1,24-24A24,24,0,0,1,128,100Zm60,24a48,48,0,1,0,48,48A48.05,48.05,0,0,0,188,124Zm0,72a24,24,0,1,1,24-24A24,24,0,0,1,188,196ZM68,124a48,48,0,1,0,48,48A48.05,48.05,0,0,0,68,124Zm0,72a24,24,0,1,1,24-24A24,24,0,0,1,68,196Z"/>
  {:else if weight === "duotone"}
    <path d="M128,112a36,36,0,1,1,36-36A36,36,0,0,1,128,112Zm60,24a36,36,0,1,0,36,36A36,36,0,0,0,188,136ZM68,136a36,36,0,1,0,36,36A36,36,0,0,0,68,136Z" opacity="0.2"/><path d="M172,76a44,44,0,1,0-44,44A44.05,44.05,0,0,0,172,76Zm-44,28a28,28,0,1,1,28-28A28,28,0,0,1,128,104Zm60,24a44,44,0,1,0,44,44A44.05,44.05,0,0,0,188,128Zm0,72a28,28,0,1,1,28-28A28,28,0,0,1,188,200ZM68,128a44,44,0,1,0,44,44A44.05,44.05,0,0,0,68,128Zm0,72a28,28,0,1,1,28-28A28,28,0,0,1,68,200Z"/>
  {:else if weight === "fill"}
    <path d="M128,120a44,44,0,1,1,44-44A44.05,44.05,0,0,1,128,120Zm60,8a44,44,0,1,0,44,44A44.05,44.05,0,0,0,188,128ZM68,128a44,44,0,1,0,44,44A44.05,44.05,0,0,0,68,128Z"/>
  {:else if weight === "light"}
    <path d="M170,76a42,42,0,1,0-42,42A42,42,0,0,0,170,76Zm-42,30a30,30,0,1,1,30-30A30,30,0,0,1,128,106Zm60,24a42,42,0,1,0,42,42A42,42,0,0,0,188,130Zm0,72a30,30,0,1,1,30-30A30,30,0,0,1,188,202ZM68,130a42,42,0,1,0,42,42A42,42,0,0,0,68,130Zm0,72a30,30,0,1,1,30-30A30,30,0,0,1,68,202Z"/>
  {:else if weight === "regular"}
    <path d="M172,76a44,44,0,1,0-44,44A44.05,44.05,0,0,0,172,76Zm-44,28a28,28,0,1,1,28-28A28,28,0,0,1,128,104Zm60,24a44,44,0,1,0,44,44A44.05,44.05,0,0,0,188,128Zm0,72a28,28,0,1,1,28-28A28,28,0,0,1,188,200ZM68,128a44,44,0,1,0,44,44A44.05,44.05,0,0,0,68,128Zm0,72a28,28,0,1,1,28-28A28,28,0,0,1,68,200Z"/>
  {:else if weight === "thin"}
    <path d="M168,76a40,40,0,1,0-40,40A40,40,0,0,0,168,76Zm-40,32a32,32,0,1,1,32-32A32,32,0,0,1,128,108Zm60,24a40,40,0,1,0,40,40A40,40,0,0,0,188,132Zm0,72a32,32,0,1,1,32-32A32,32,0,0,1,188,204ZM68,132a40,40,0,1,0,40,40A40,40,0,0,0,68,132Zm0,72a32,32,0,1,1,32-32A32,32,0,0,1,68,204Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>