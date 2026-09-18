import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GooglePodcastsLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <GooglePodcastsLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GooglePodcastsLogo: Component<IconComponentProps, {}, "">;
type GooglePodcastsLogo = ReturnType<typeof GooglePodcastsLogo>;
export default GooglePodcastsLogo;
