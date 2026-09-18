import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CalendarDotIcon` instead.
 *
 * @example
 * ```svelte
 * <CalendarDot color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CalendarDot: Component<IconComponentProps, {}, "">;
type CalendarDot = ReturnType<typeof CalendarDot>;
export default CalendarDot;
