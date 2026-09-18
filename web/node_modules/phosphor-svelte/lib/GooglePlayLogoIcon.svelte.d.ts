import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <GooglePlayLogoIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GooglePlayLogoIcon: Component<IconComponentProps, {}, "">;
type GooglePlayLogoIcon = ReturnType<typeof GooglePlayLogoIcon>;
export default GooglePlayLogoIcon;
