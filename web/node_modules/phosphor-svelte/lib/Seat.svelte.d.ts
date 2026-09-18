import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SeatIcon` instead.
 *
 * @example
 * ```svelte
 * <Seat color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Seat: Component<IconComponentProps, {}, "">;
type Seat = ReturnType<typeof Seat>;
export default Seat;
