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
    <path d="M208,36H96a68,68,0,0,0,0,136h36v36a12,12,0,0,0,24,0V60h16V208a12,12,0,0,0,24,0V60h12a12,12,0,0,0,0-24ZM132,148H96a44,44,0,0,1,0-88h36Z"/>
  {:else if weight === "duotone"}
    <path d="M144,48V160H96A56,56,0,0,1,96,48Z" opacity="0.2"/><path d="M208,40H96a64,64,0,0,0,0,128h40v40a8,8,0,0,0,16,0V56h24V208a8,8,0,0,0,16,0V56h16a8,8,0,0,0,0-16ZM136,152H96a48,48,0,0,1,0-96h40Z"/>
  {:else if weight === "fill"}
    <path d="M216,48a8,8,0,0,1-8,8H192V208a8,8,0,0,1-16,0V56H152V208a8,8,0,0,1-16,0V168H96A64,64,0,0,1,96,40H208A8,8,0,0,1,216,48Z"/>
  {:else if weight === "light"}
    <path d="M208,42H96a62,62,0,0,0,0,124h42v42a6,6,0,0,0,12,0V54h28V208a6,6,0,0,0,12,0V54h18a6,6,0,0,0,0-12ZM138,154H96A50,50,0,0,1,96,54h42Z"/>
  {:else if weight === "regular"}
    <path d="M208,40H96a64,64,0,0,0,0,128h40v40a8,8,0,0,0,16,0V56h24V208a8,8,0,0,0,16,0V56h16a8,8,0,0,0,0-16ZM136,152H96a48,48,0,0,1,0-96h40Z"/>
  {:else if weight === "thin"}
    <path d="M208,44H96a60,60,0,0,0,0,120h44v44a4,4,0,0,0,8,0V52h32V208a4,4,0,0,0,8,0V52h20a4,4,0,0,0,0-8ZM140,156H96A52,52,0,0,1,96,52h44Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>