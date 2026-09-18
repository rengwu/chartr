import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TicketIcon` instead.
 *
 * @example
 * ```svelte
 * <Ticket color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Ticket: Component<IconComponentProps, {}, "">;
type Ticket = ReturnType<typeof Ticket>;
export default Ticket;
