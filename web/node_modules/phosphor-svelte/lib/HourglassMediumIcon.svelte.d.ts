import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <HourglassMediumIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HourglassMediumIcon: Component<IconComponentProps, {}, "">;
type HourglassMediumIcon = ReturnType<typeof HourglassMediumIcon>;
export default HourglassMediumIcon;
