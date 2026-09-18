import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrendDownIcon` instead.
 *
 * @example
 * ```svelte
 * <TrendDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrendDown: Component<IconComponentProps, {}, "">;
type TrendDown = ReturnType<typeof TrendDown>;
export default TrendDown;
