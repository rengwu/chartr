import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StrategyIcon` instead.
 *
 * @example
 * ```svelte
 * <Strategy color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Strategy: Component<IconComponentProps, {}, "">;
type Strategy = ReturnType<typeof Strategy>;
export default Strategy;
