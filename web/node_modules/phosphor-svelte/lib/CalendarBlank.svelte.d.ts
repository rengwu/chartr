import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CalendarBlankIcon` instead.
 *
 * @example
 * ```svelte
 * <CalendarBlank color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CalendarBlank: Component<IconComponentProps, {}, "">;
type CalendarBlank = ReturnType<typeof CalendarBlank>;
export default CalendarBlank;
