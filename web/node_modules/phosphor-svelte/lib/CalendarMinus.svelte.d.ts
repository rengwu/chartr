import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CalendarMinusIcon` instead.
 *
 * @example
 * ```svelte
 * <CalendarMinus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CalendarMinus: Component<IconComponentProps, {}, "">;
type CalendarMinus = ReturnType<typeof CalendarMinus>;
export default CalendarMinus;
