import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FacebookLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <FacebookLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FacebookLogo: Component<IconComponentProps, {}, "">;
type FacebookLogo = ReturnType<typeof FacebookLogo>;
export default FacebookLogo;
