import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChartPieIcon` instead.
 *
 * @example
 * ```svelte
 * <ChartPie color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChartPie: Component<IconComponentProps, {}, "">;
type ChartPie = ReturnType<typeof ChartPie>;
export default ChartPie;
