import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TwitterLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <TwitterLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TwitterLogo: Component<IconComponentProps, {}, "">;
type TwitterLogo = ReturnType<typeof TwitterLogo>;
export default TwitterLogo;
