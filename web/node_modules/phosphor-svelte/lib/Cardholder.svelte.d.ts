import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CardholderIcon` instead.
 *
 * @example
 * ```svelte
 * <Cardholder color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cardholder: Component<IconComponentProps, {}, "">;
type Cardholder = ReturnType<typeof Cardholder>;
export default Cardholder;
