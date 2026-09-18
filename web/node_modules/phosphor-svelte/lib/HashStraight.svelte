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
    <path d="M216,148H172V108h44a12,12,0,0,0,0-24H172V40a12,12,0,0,0-24,0V84H108V40a12,12,0,0,0-24,0V84H40a12,12,0,0,0,0,24H84v40H40a12,12,0,0,0,0,24H84v44a12,12,0,0,0,24,0V172h40v44a12,12,0,0,0,24,0V172h44a12,12,0,0,0,0-24Zm-108,0V108h40v40Z"/>
  {:else if weight === "duotone"}
    <path d="M160,96v64H96V96Z" opacity="0.2"/><path d="M216,152H168V104h48a8,8,0,0,0,0-16H168V40a8,8,0,0,0-16,0V88H104V40a8,8,0,0,0-16,0V88H40a8,8,0,0,0,0,16H88v48H40a8,8,0,0,0,0,16H88v48a8,8,0,0,0,16,0V168h48v48a8,8,0,0,0,16,0V168h48a8,8,0,0,0,0-16Zm-112,0V104h48v48Z"/>
  {:else if weight === "fill"}
    <path d="M112,112h32v32H112ZM224,48V208a16,16,0,0,1-16,16H48a16,16,0,0,1-16-16V48A16,16,0,0,1,48,32H208A16,16,0,0,1,224,48Zm-64,96V112h32a8,8,0,0,0,0-16H160V64a8,8,0,0,0-16,0V96H112V64a8,8,0,0,0-16,0V96H64a8,8,0,0,0,0,16H96v32H64a8,8,0,0,0,0,16H96v32a8,8,0,0,0,16,0V160h32v32a8,8,0,0,0,16,0V160h32a8,8,0,0,0,0-16Z"/>
  {:else if weight === "light"}
    <path d="M216,154H166V102h50a6,6,0,0,0,0-12H166V40a6,6,0,0,0-12,0V90H102V40a6,6,0,0,0-12,0V90H40a6,6,0,0,0,0,12H90v52H40a6,6,0,0,0,0,12H90v50a6,6,0,0,0,12,0V166h52v50a6,6,0,0,0,12,0V166h50a6,6,0,0,0,0-12Zm-114,0V102h52v52Z"/>
  {:else if weight === "regular"}
    <path d="M216,152H168V104h48a8,8,0,0,0,0-16H168V40a8,8,0,0,0-16,0V88H104V40a8,8,0,0,0-16,0V88H40a8,8,0,0,0,0,16H88v48H40a8,8,0,0,0,0,16H88v48a8,8,0,0,0,16,0V168h48v48a8,8,0,0,0,16,0V168h48a8,8,0,0,0,0-16Zm-112,0V104h48v48Z"/>
  {:else if weight === "thin"}
    <path d="M216,156H164V100h52a4,4,0,0,0,0-8H164V40a4,4,0,0,0-8,0V92H100V40a4,4,0,0,0-8,0V92H40a4,4,0,0,0,0,8H92v56H40a4,4,0,0,0,0,8H92v52a4,4,0,0,0,8,0V164h56v52a4,4,0,0,0,8,0V164h52a4,4,0,0,0,0-8Zm-116,0V100h56v56Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>