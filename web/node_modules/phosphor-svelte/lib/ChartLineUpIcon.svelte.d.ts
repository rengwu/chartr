import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ChartLineUpIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChartLineUpIcon: Component<IconComponentProps, {}, "">;
type ChartLineUpIcon = ReturnType<typeof ChartLineUpIcon>;
export default ChartLineUpIcon;
