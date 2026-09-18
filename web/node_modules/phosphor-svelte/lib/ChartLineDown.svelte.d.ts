import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChartLineDownIcon` instead.
 *
 * @example
 * ```svelte
 * <ChartLineDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChartLineDown: Component<IconComponentProps, {}, "">;
type ChartLineDown = ReturnType<typeof ChartLineDown>;
export default ChartLineDown;
