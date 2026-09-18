import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FigmaLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <FigmaLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FigmaLogo: Component<IconComponentProps, {}, "">;
type FigmaLogo = ReturnType<typeof FigmaLogo>;
export default FigmaLogo;
