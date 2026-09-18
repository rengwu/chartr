import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowCircleDownIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowCircleDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowCircleDown: Component<IconComponentProps, {}, "">;
type ArrowCircleDown = ReturnType<typeof ArrowCircleDown>;
export default ArrowCircleDown;
