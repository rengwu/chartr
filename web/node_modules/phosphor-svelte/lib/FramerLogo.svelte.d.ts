import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FramerLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <FramerLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FramerLogo: Component<IconComponentProps, {}, "">;
type FramerLogo = ReturnType<typeof FramerLogo>;
export default FramerLogo;
