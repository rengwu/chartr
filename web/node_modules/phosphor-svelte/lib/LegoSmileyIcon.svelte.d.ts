import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <LegoSmileyIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LegoSmileyIcon: Component<IconComponentProps, {}, "">;
type LegoSmileyIcon = ReturnType<typeof LegoSmileyIcon>;
export default LegoSmileyIcon;
