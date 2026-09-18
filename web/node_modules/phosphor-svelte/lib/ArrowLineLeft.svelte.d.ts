import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowLineLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowLineLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowLineLeft: Component<IconComponentProps, {}, "">;
type ArrowLineLeft = ReturnType<typeof ArrowLineLeft>;
export default ArrowLineLeft;
