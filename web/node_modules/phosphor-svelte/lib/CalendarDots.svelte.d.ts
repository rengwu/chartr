import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CalendarDotsIcon` instead.
 *
 * @example
 * ```svelte
 * <CalendarDots color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CalendarDots: Component<IconComponentProps, {}, "">;
type CalendarDots = ReturnType<typeof CalendarDots>;
export default CalendarDots;
