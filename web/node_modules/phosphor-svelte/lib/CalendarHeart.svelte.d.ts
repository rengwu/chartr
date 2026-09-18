import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CalendarHeartIcon` instead.
 *
 * @example
 * ```svelte
 * <CalendarHeart color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CalendarHeart: Component<IconComponentProps, {}, "">;
type CalendarHeart = ReturnType<typeof CalendarHeart>;
export default CalendarHeart;
