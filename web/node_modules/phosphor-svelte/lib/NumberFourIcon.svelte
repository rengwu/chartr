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
    <path d="M184,156H172V48a12,12,0,0,0-21.37-7.5l-96,120A12,12,0,0,0,64,180h84v28a12,12,0,0,0,24,0V180h12a12,12,0,0,0,0-24Zm-36,0H89l59-73.79Z"/>
  {:else if weight === "duotone"}
    <path d="M216,40V216a16,16,0,0,1-16,16H56a16,16,0,0,1-16-16V40A16,16,0,0,1,56,24H200A16,16,0,0,1,216,40Z" opacity="0.2"/><path d="M184,160H168V48a8,8,0,0,0-14.25-5l-96,120A8,8,0,0,0,64,176h88v32a8,8,0,0,0,16,0V176h16a8,8,0,0,0,0-16Zm-32,0H80.64L152,70.81Z"/>
  {:else if weight === "fill"}
    <path d="M104.65,144,144,94.81V144ZM216,40V216a16,16,0,0,1-16,16H56a16,16,0,0,1-16-16V40A16,16,0,0,1,56,24H200A16,16,0,0,1,216,40ZM184,152a8,8,0,0,0-8-8H160V72a8,8,0,0,0-14.25-5l-64,80A8,8,0,0,0,88,160h56v24a8,8,0,0,0,16,0V160h16A8,8,0,0,0,184,152Z"/>
  {:else if weight === "light"}
    <path d="M184,162H166V48a6,6,0,0,0-10.69-3.75l-96,120A6,6,0,0,0,64,174h90v34a6,6,0,0,0,12,0V174h18a6,6,0,0,0,0-12Zm-30,0H76.48L154,65.1Z"/>
  {:else if weight === "regular"}
    <path d="M184,160H168V48a8,8,0,0,0-14.25-5l-96,120A8,8,0,0,0,64,176h88v32a8,8,0,0,0,16,0V176h16a8,8,0,0,0,0-16Zm-32,0H80.64L152,70.81Z"/>
  {:else if weight === "thin"}
    <path d="M184,164H164V48a4,4,0,0,0-7.12-2.5l-96,120A4,4,0,0,0,64,172h92v36a4,4,0,0,0,8,0V172h20a4,4,0,0,0,0-8Zm-28,0H72.32L156,59.4Z"/>
  {:else}
    {(console.error('Unsupported icon weight. Choose from "thin", "light", "regular", "bold", "fill", or "duotone".'), "")}
  {/if}
</svg>