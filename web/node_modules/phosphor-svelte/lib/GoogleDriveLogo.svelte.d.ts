import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GoogleDriveLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <GoogleDriveLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GoogleDriveLogo: Component<IconComponentProps, {}, "">;
type GoogleDriveLogo = ReturnType<typeof GoogleDriveLogo>;
export default GoogleDriveLogo;
