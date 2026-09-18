import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ContactlessPaymentIcon` instead.
 *
 * @example
 * ```svelte
 * <ContactlessPayment color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ContactlessPayment: Component<IconComponentProps, {}, "">;
type ContactlessPayment = ReturnType<typeof ContactlessPayment>;
export default ContactlessPayment;
