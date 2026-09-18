import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AngularLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <AngularLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AngularLogo: Component<IconComponentProps, {}, "">;
type AngularLogo = ReturnType<typeof AngularLogo>;
export default AngularLogo;
