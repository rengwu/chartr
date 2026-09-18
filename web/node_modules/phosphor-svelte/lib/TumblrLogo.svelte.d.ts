import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TumblrLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <TumblrLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TumblrLogo: Component<IconComponentProps, {}, "">;
type TumblrLogo = ReturnType<typeof TumblrLogo>;
export default TumblrLogo;
