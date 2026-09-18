import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChartDonutIcon` instead.
 *
 * @example
 * ```svelte
 * <ChartDonut color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChartDonut: Component<IconComponentProps, {}, "">;
type ChartDonut = ReturnType<typeof ChartDonut>;
export default ChartDonut;
