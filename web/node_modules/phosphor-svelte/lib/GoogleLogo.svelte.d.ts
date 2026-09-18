import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GoogleLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <GoogleLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GoogleLogo: Component<IconComponentProps, {}, "">;
type GoogleLogo = ReturnType<typeof GoogleLogo>;
export default GoogleLogo;
