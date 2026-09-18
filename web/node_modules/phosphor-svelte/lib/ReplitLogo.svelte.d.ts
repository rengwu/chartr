import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ReplitLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <ReplitLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ReplitLogo: Component<IconComponentProps, {}, "">;
type ReplitLogo = ReturnType<typeof ReplitLogo>;
export default ReplitLogo;
