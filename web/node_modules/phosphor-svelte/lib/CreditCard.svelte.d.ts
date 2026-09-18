import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CreditCardIcon` instead.
 *
 * @example
 * ```svelte
 * <CreditCard color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CreditCard: Component<IconComponentProps, {}, "">;
type CreditCard = ReturnType<typeof CreditCard>;
export default CreditCard;
