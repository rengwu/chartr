import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `InvoiceIcon` instead.
 *
 * @example
 * ```svelte
 * <Invoice color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Invoice: Component<IconComponentProps, {}, "">;
type Invoice = ReturnType<typeof Invoice>;
export default Invoice;
