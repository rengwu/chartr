import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ArrowLineDownIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowLineDownIcon: Component<IconComponentProps, {}, "">;
type ArrowLineDownIcon = ReturnType<typeof ArrowLineDownIcon>;
export default ArrowLineDownIcon;
