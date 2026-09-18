import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatLineDownIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatLineDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatLineDown: Component<IconComponentProps, {}, "">;
type ArrowFatLineDown = ReturnType<typeof ArrowFatLineDown>;
export default ArrowFatLineDown;
