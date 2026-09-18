import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ThermometerSimpleIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ThermometerSimpleIcon: Component<IconComponentProps, {}, "">;
type ThermometerSimpleIcon = ReturnType<typeof ThermometerSimpleIcon>;
export default ThermometerSimpleIcon;
