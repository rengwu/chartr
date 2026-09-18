import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ThermometerIcon` instead.
 *
 * @example
 * ```svelte
 * <Thermometer color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Thermometer: Component<IconComponentProps, {}, "">;
type Thermometer = ReturnType<typeof Thermometer>;
export default Thermometer;
