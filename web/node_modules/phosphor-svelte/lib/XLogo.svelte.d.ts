import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `XLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <XLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const XLogo: Component<IconComponentProps, {}, "">;
type XLogo = ReturnType<typeof XLogo>;
export default XLogo;
