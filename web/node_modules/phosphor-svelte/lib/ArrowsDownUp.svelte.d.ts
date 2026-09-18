import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsDownUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsDownUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsDownUp: Component<IconComponentProps, {}, "">;
type ArrowsDownUp = ReturnType<typeof ArrowsDownUp>;
export default ArrowsDownUp;
