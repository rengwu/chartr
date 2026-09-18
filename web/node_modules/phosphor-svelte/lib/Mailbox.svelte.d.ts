import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MailboxIcon` instead.
 *
 * @example
 * ```svelte
 * <Mailbox color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Mailbox: Component<IconComponentProps, {}, "">;
type Mailbox = ReturnType<typeof Mailbox>;
export default Mailbox;
