import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyCircleDollarIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyCircleDollar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyCircleDollar: Component<IconComponentProps, {}, "">;
type CurrencyCircleDollar = ReturnType<typeof CurrencyCircleDollar>;
export default CurrencyCircleDollar;
