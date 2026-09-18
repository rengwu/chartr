import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ContactlessPaymentIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ContactlessPaymentIcon: Component<IconComponentProps, {}, "">;
type ContactlessPaymentIcon = ReturnType<typeof ContactlessPaymentIcon>;
export default ContactlessPaymentIcon;
