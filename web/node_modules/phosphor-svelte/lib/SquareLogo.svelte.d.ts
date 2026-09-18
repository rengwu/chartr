import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SquareLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <SquareLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SquareLogo: Component<IconComponentProps, {}, "">;
type SquareLogo = ReturnType<typeof SquareLogo>;
export default SquareLogo;
