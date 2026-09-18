import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WindowsLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <WindowsLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WindowsLogo: Component<IconComponentProps, {}, "">;
type WindowsLogo = ReturnType<typeof WindowsLogo>;
export default WindowsLogo;
