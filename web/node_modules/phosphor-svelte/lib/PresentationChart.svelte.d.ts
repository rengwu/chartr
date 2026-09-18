import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PresentationChartIcon` instead.
 *
 * @example
 * ```svelte
 * <PresentationChart color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PresentationChart: Component<IconComponentProps, {}, "">;
type PresentationChart = ReturnType<typeof PresentationChart>;
export default PresentationChart;
