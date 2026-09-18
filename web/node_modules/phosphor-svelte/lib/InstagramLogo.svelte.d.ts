import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `InstagramLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <InstagramLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const InstagramLogo: Component<IconComponentProps, {}, "">;
type InstagramLogo = ReturnType<typeof InstagramLogo>;
export default InstagramLogo;
