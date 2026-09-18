import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CalendarCheckIcon` instead.
 *
 * @example
 * ```svelte
 * <CalendarCheck color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CalendarCheck: Component<IconComponentProps, {}, "">;
type CalendarCheck = ReturnType<typeof CalendarCheck>;
export default CalendarCheck;
