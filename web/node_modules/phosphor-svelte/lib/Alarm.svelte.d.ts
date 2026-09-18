import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlarmIcon` instead.
 *
 * @example
 * ```svelte
 * <Alarm color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Alarm: Component<IconComponentProps, {}, "">;
type Alarm = ReturnType<typeof Alarm>;
export default Alarm;
