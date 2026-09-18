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
    <path d="M188,72a60,60,0,1,0-72,58.79V232a12,12,0,0,0,24,0V130.79A60.09,60.09,0,0,0,188,72Zm-60,36a36,36,0,1,1,36-36A36,36,0,0,1,128,108Z"/>
  {:else if weight === "duotone"}
    <path d="M176,72a48,48,0,1,1-48-48A48,48,0,0,1,176,72Z" opacity="0.2"/><path d="M184,72a56,56,0,1,0-64,55.42V232a8,8,0,0,0,16,0V127.42A56.09,56.09,0,0,0,184,72Zm-56,40a40,40,0,1,1,40-40A40,40,0,0,1,128,112Z"/>
  {:else if weight === "fill"}
    <path d="M136,127.42V232a8,8,0,0,1-16,0V127.42a56,56,0,1,1,16,0Z"/>
  {:else if weight === "light"}
    <path d="M182,72a54,54,0,1,0-60,53.66V232a6,6,0,0,0,12,0V125.66A54.07,54.07,0,0,0,182,72Zm-54,42a42,42,0,1,1,42-42A42,42,0,0,1,128,114Z"/>
  {:else if weight === "regular"}
    <path d="M184,72a56,56,0,1,0-64,55.42V232a8,8,0,0,0,16,0V127.42A56.09,56.09,0,0,0,184,72Zm-56,40a40,40,0,1,1,40-40A40,40,0,0,1,128,112Z"/>
  {:else if weight === "thin"}
    <path d="M180,72a52,52,0,1,0-56,51.83V232a4,4,0,0,0,8,0V123.83A52.05,52.05,0,0,0,180,72Zm-52,44a44,44,0,1,1,44-44A44.05,44.05,0,0,1,128,116Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>