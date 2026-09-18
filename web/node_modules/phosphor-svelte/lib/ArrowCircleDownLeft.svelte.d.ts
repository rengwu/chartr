import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowCircleDownLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowCircleDownLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowCircleDownLeft: Component<IconComponentProps, {}, "">;
type ArrowCircleDownLeft = ReturnType<typeof ArrowCircleDownLeft>;
export default ArrowCircleDownLeft;
