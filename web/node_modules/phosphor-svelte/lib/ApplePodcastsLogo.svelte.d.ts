import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ApplePodcastsLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <ApplePodcastsLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ApplePodcastsLogo: Component<IconComponentProps, {}, "">;
type ApplePodcastsLogo = ReturnType<typeof ApplePodcastsLogo>;
export default ApplePodcastsLogo;
