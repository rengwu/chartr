import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AndroidLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <AndroidLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AndroidLogo: Component<IconComponentProps, {}, "">;
type AndroidLogo = ReturnType<typeof AndroidLogo>;
export default AndroidLogo;
