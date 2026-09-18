import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ReceiptXIcon` instead.
 *
 * @example
 * ```svelte
 * <ReceiptX color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ReceiptX: Component<IconComponentProps, {}, "">;
type ReceiptX = ReturnType<typeof ReceiptX>;
export default ReceiptX;
