import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NyTimesLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <NyTimesLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NyTimesLogo: Component<IconComponentProps, {}, "">;
type NyTimesLogo = ReturnType<typeof NyTimesLogo>;
export default NyTimesLogo;
