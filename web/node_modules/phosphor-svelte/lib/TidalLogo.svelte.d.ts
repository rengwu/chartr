import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TidalLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <TidalLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TidalLogo: Component<IconComponentProps, {}, "">;
type TidalLogo = ReturnType<typeof TidalLogo>;
export default TidalLogo;
