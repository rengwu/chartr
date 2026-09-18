import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChampagneIcon` instead.
 *
 * @example
 * ```svelte
 * <Champagne color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Champagne: Component<IconComponentProps, {}, "">;
type Champagne = ReturnType<typeof Champagne>;
export default Champagne;
