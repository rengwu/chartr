import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ArrowArcRightIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowArcRightIcon: Component<IconComponentProps, {}, "">;
type ArrowArcRightIcon = ReturnType<typeof ArrowArcRightIcon>;
export default ArrowArcRightIcon;
