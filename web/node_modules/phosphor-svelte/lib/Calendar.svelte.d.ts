import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CalendarIcon` instead.
 *
 * @example
 * ```svelte
 * <Calendar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Calendar: Component<IconComponentProps, {}, "">;
type Calendar = ReturnType<typeof Calendar>;
export default Calendar;
