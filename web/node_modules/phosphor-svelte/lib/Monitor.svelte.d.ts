import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MonitorIcon` instead.
 *
 * @example
 * ```svelte
 * <Monitor color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Monitor: Component<IconComponentProps, {}, "">;
type Monitor = ReturnType<typeof Monitor>;
export default Monitor;
