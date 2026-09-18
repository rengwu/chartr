import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HockeyIcon` instead.
 *
 * @example
 * ```svelte
 * <Hockey color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Hockey: Component<IconComponentProps, {}, "">;
type Hockey = ReturnType<typeof Hockey>;
export default Hockey;
