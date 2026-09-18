import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DevToLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <DevToLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DevToLogo: Component<IconComponentProps, {}, "">;
type DevToLogo = ReturnType<typeof DevToLogo>;
export default DevToLogo;
