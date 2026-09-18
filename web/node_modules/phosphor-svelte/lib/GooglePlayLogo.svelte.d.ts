import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GooglePlayLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <GooglePlayLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GooglePlayLogo: Component<IconComponentProps, {}, "">;
type GooglePlayLogo = ReturnType<typeof GooglePlayLogo>;
export default GooglePlayLogo;
