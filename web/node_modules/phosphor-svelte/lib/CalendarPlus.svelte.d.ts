import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CalendarPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <CalendarPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CalendarPlus: Component<IconComponentProps, {}, "">;
type CalendarPlus = ReturnType<typeof CalendarPlus>;
export default CalendarPlus;
