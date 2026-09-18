import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MonitorPlayIcon` instead.
 *
 * @example
 * ```svelte
 * <MonitorPlay color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MonitorPlay: Component<IconComponentProps, {}, "">;
type MonitorPlay = ReturnType<typeof MonitorPlay>;
export default MonitorPlay;
