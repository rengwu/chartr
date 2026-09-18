import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AppleLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <AppleLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AppleLogo: Component<IconComponentProps, {}, "">;
type AppleLogo = ReturnType<typeof AppleLogo>;
export default AppleLogo;
