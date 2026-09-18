import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SpotifyLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <SpotifyLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SpotifyLogo: Component<IconComponentProps, {}, "">;
type SpotifyLogo = ReturnType<typeof SpotifyLogo>;
export default SpotifyLogo;
