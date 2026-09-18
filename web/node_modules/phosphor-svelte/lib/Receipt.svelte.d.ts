import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ReceiptIcon` instead.
 *
 * @example
 * ```svelte
 * <Receipt color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Receipt: Component<IconComponentProps, {}, "">;
type Receipt = ReturnType<typeof Receipt>;
export default Receipt;
