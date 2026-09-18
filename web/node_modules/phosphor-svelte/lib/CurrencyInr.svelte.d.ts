import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyInrIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyInr color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyInr: Component<IconComponentProps, {}, "">;
type CurrencyInr = ReturnType<typeof CurrencyInr>;
export default CurrencyInr;
