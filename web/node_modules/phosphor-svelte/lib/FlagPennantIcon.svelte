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
    <path d="M243.94,92.67l-184-64A12,12,0,0,0,44,40V216a12,12,0,0,0,24,0V176.53l175.94-61.2a12,12,0,0,0,0-22.66ZM68,151.12V56.88L203.47,104Z"/>
  {:else if weight === "duotone"}
    <path d="M240,104,56,168V40Z" opacity="0.2"/><path d="M242.63,96.44l-184-64A8,8,0,0,0,48,40V216a8,8,0,0,0,16,0V173.69l178.63-62.13a8,8,0,0,0,0-15.12ZM64,156.75V51.25L215.65,104Z"/>
  {:else if weight === "fill"}
    <path d="M248,104a8,8,0,0,1-5.37,7.56L64,173.69V216a8,8,0,0,1-16,0V40a8,8,0,0,1,10.63-7.56l184,64A8,8,0,0,1,248,104Z"/>
  {:else if weight === "light"}
    <path d="M242,98.33l-184-64A6,6,0,0,0,50,40V216a6,6,0,0,0,12,0V172.27l180-62.6a6,6,0,0,0,0-11.34ZM62,159.56V48.44L221.74,104Z"/>
  {:else if weight === "regular"}
    <path d="M242.63,96.44l-184-64A8,8,0,0,0,48,40V216a8,8,0,0,0,16,0V173.69l178.63-62.13a8,8,0,0,0,0-15.12ZM64,156.75V51.25L215.65,104Z"/>
  {:else if weight === "thin"}
    <path d="M241.31,100.22l-184-64A4,4,0,0,0,52,40V216a4,4,0,0,0,8,0V170.84l181.31-63.06a4,4,0,0,0,0-7.56ZM60,162.37V45.63L227.82,104Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>