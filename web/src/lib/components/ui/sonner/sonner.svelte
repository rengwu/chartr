<script lang="ts">
  import { Toaster as Sonner, type ToasterProps as SonnerProps } from "svelte-sonner";

  // The one toast surface for the whole cockpit — mounted once at the app root
  // (App.svelte) and fed by `toast(...)` from anywhere. It replaces the inline
  // status/alert lines the sidebar and the register forms used to paint in place.
  //
  // The chrome boots dark and never toggles (index.html), so the theme is pinned
  // rather than watched. The token mapping ties every toast to the same
  // popover/border/destructive tokens the rest of the surface uses; the
  // center-offset and the destructive error tint live in app.css, which is the
  // only place the `[data-x-position]` / `[data-type]` selectors can be reached.
  let restProps: SonnerProps = $props();
</script>

<Sonner
  theme="dark"
  class="chartr-toaster"
  position="top-center"
  richColors={false}
  style="--normal-bg: var(--popover); --normal-text: var(--popover-foreground); --normal-border: var(--border); --border-radius: var(--radius);"
  {...restProps}
/>
