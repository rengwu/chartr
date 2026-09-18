import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RedditLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <RedditLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RedditLogo: Component<IconComponentProps, {}, "">;
type RedditLogo = ReturnType<typeof RedditLogo>;
export default RedditLogo;
