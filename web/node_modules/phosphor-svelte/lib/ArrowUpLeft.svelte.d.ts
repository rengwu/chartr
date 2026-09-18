import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowUpLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowUpLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowUpLeft: Component<IconComponentProps, {}, "">;
type ArrowUpLeft = ReturnType<typeof ArrowUpLeft>;
export default ArrowUpLeft;
