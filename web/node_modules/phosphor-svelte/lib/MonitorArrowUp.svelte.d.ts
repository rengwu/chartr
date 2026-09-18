import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MonitorArrowUpIcon` instead.
 *
 * @example
 * ```svelte
 * <MonitorArrowUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MonitorArrowUp: Component<IconComponentProps, {}, "">;
type MonitorArrowUp = ReturnType<typeof MonitorArrowUp>;
export default MonitorArrowUp;
