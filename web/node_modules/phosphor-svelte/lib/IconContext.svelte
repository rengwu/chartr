<script>
  import { setIconContext } from "./context";

  let { children, values } = $props();

  setIconContext(values);
</script>

{@render children?.()}
