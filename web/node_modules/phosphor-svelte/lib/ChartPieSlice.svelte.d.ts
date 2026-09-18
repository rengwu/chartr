import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChartPieSliceIcon` instead.
 *
 * @example
 * ```svelte
 * <ChartPieSlice color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChartPieSlice: Component<IconComponentProps, {}, "">;
type ChartPieSlice = ReturnType<typeof ChartPieSlice>;
export default ChartPieSlice;
