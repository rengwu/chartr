import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CalendarXIcon` instead.
 *
 * @example
 * ```svelte
 * <CalendarX color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CalendarX: Component<IconComponentProps, {}, "">;
type CalendarX = ReturnType<typeof CalendarX>;
export default CalendarX;
