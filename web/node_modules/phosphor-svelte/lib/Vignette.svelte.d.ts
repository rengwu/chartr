import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VignetteIcon` instead.
 *
 * @example
 * ```svelte
 * <Vignette color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Vignette: Component<IconComponentProps, {}, "">;
type Vignette = ReturnType<typeof Vignette>;
export default Vignette;
