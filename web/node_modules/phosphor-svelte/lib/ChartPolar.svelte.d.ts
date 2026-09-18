import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChartPolarIcon` instead.
 *
 * @example
 * ```svelte
 * <ChartPolar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChartPolar: Component<IconComponentProps, {}, "">;
type ChartPolar = ReturnType<typeof ChartPolar>;
export default ChartPolar;
