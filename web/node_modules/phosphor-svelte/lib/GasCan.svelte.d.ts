import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GasCanIcon` instead.
 *
 * @example
 * ```svelte
 * <GasCan color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GasCan: Component<IconComponentProps, {}, "">;
type GasCan = ReturnType<typeof GasCan>;
export default GasCan;
