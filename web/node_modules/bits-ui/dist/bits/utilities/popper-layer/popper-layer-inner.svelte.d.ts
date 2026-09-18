import type { PopperLayerImplProps } from "./types.js";
type $$ComponentProps = Omit<PopperLayerImplProps, "open" | "children" | "shouldRender"> & {
    enabled: boolean;
    contentPointerEvents?: "auto" | "none";
};
declare const PopperLayerInner: import("svelte").Component<$$ComponentProps, {}, "">;
type PopperLayerInner = ReturnType<typeof PopperLayerInner>;
export default PopperLayerInner;
