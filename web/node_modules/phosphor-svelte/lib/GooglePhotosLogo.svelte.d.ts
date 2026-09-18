import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GooglePhotosLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <GooglePhotosLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GooglePhotosLogo: Component<IconComponentProps, {}, "">;
type GooglePhotosLogo = ReturnType<typeof GooglePhotosLogo>;
export default GooglePhotosLogo;
