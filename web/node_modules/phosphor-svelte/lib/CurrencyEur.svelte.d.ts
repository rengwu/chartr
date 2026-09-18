import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyEurIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyEur color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyEur: Component<IconComponentProps, {}, "">;
type CurrencyEur = ReturnType<typeof CurrencyEur>;
export default CurrencyEur;
