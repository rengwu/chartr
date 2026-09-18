import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StripeLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <StripeLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const StripeLogo: Component<IconComponentProps, {}, "">;
type StripeLogo = ReturnType<typeof StripeLogo>;
export default StripeLogo;
