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
    <path d="M252,56a12,12,0,0,1-12,12H196v36a12,12,0,0,1-12,12H140v36a12,12,0,0,1-12,12H84v36a12,12,0,0,1-12,12H16a12,12,0,0,1,0-24H60V152a12,12,0,0,1,12-12h44V104a12,12,0,0,1,12-12h44V56a12,12,0,0,1,12-12h56A12,12,0,0,1,252,56Z"/>
  {:else if weight === "duotone"}
    <path d="M240,56V184a16,16,0,0,1-16,16H72V152h56V104h56V56Z" opacity="0.2"/><path d="M248,56a8,8,0,0,1-8,8H192v40a8,8,0,0,1-8,8H136v40a8,8,0,0,1-8,8H80v40a8,8,0,0,1-8,8H16a8,8,0,0,1,0-16H64V152a8,8,0,0,1,8-8h48V104a8,8,0,0,1,8-8h48V56a8,8,0,0,1,8-8h56A8,8,0,0,1,248,56Z"/>
  {:else if weight === "fill"}
    <path d="M240,56V200a8,8,0,0,1-8,8H8a8,8,0,0,1,0-16H56V152a8,8,0,0,1,8-8h48V104a8,8,0,0,1,8-8h48V56a8,8,0,0,1,8-8h56A8,8,0,0,1,240,56Z"/>
  {:else if weight === "light"}
    <path d="M246,56a6,6,0,0,1-6,6H190v42a6,6,0,0,1-6,6H134v42a6,6,0,0,1-6,6H78v42a6,6,0,0,1-6,6H16a6,6,0,0,1,0-12H66V152a6,6,0,0,1,6-6h50V104a6,6,0,0,1,6-6h50V56a6,6,0,0,1,6-6h56A6,6,0,0,1,246,56Z"/>
  {:else if weight === "regular"}
    <path d="M248,56a8,8,0,0,1-8,8H192v40a8,8,0,0,1-8,8H136v40a8,8,0,0,1-8,8H80v40a8,8,0,0,1-8,8H16a8,8,0,0,1,0-16H64V152a8,8,0,0,1,8-8h48V104a8,8,0,0,1,8-8h48V56a8,8,0,0,1,8-8h56A8,8,0,0,1,248,56Z"/>
  {:else if weight === "thin"}
    <path d="M244,56a4,4,0,0,1-4,4H188v44a4,4,0,0,1-4,4H132v44a4,4,0,0,1-4,4H76v44a4,4,0,0,1-4,4H16a4,4,0,0,1,0-8H68V152a4,4,0,0,1,4-4h52V104a4,4,0,0,1,4-4h52V56a4,4,0,0,1,4-4h56A4,4,0,0,1,244,56Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>