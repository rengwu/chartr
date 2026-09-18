import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChartBarIcon` instead.
 *
 * @example
 * ```svelte
 * <ChartBar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChartBar: Component<IconComponentProps, {}, "">;
type ChartBar = ReturnType<typeof ChartBar>;
export default ChartBar;
