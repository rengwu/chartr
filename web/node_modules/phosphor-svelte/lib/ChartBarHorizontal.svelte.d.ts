import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChartBarHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <ChartBarHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChartBarHorizontal: Component<IconComponentProps, {}, "">;
type ChartBarHorizontal = ReturnType<typeof ChartBarHorizontal>;
export default ChartBarHorizontal;
