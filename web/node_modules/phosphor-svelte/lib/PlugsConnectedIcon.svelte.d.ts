import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <PlugsConnectedIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PlugsConnectedIcon: Component<IconComponentProps, {}, "">;
type PlugsConnectedIcon = ReturnType<typeof PlugsConnectedIcon>;
export default PlugsConnectedIcon;
