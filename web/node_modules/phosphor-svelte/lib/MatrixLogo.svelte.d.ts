import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MatrixLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MatrixLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MatrixLogo: Component<IconComponentProps, {}, "">;
type MatrixLogo = ReturnType<typeof MatrixLogo>;
export default MatrixLogo;
