import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowDownIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowDown: Component<IconComponentProps, {}, "">;
type ArrowDown = ReturnType<typeof ArrowDown>;
export default ArrowDown;
