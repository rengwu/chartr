import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyDollarIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyDollar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyDollar: Component<IconComponentProps, {}, "">;
type CurrencyDollar = ReturnType<typeof CurrencyDollar>;
export default CurrencyDollar;
