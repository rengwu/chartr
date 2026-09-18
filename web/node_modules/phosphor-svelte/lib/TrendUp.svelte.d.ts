import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrendUpIcon` instead.
 *
 * @example
 * ```svelte
 * <TrendUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrendUp: Component<IconComponentProps, {}, "">;
type TrendUp = ReturnType<typeof TrendUp>;
export default TrendUp;
