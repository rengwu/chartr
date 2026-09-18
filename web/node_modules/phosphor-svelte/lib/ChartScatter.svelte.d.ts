import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChartScatterIcon` instead.
 *
 * @example
 * ```svelte
 * <ChartScatter color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChartScatter: Component<IconComponentProps, {}, "">;
type ChartScatter = ReturnType<typeof ChartScatter>;
export default ChartScatter;
