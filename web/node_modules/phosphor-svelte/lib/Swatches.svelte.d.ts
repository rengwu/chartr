import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SwatchesIcon` instead.
 *
 * @example
 * ```svelte
 * <Swatches color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Swatches: Component<IconComponentProps, {}, "">;
type Swatches = ReturnType<typeof Swatches>;
export default Swatches;
