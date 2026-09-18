import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CodaLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <CodaLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CodaLogo: Component<IconComponentProps, {}, "">;
type CodaLogo = ReturnType<typeof CodaLogo>;
export default CodaLogo;
