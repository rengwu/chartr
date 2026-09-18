import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyEthIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyEth color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyEth: Component<IconComponentProps, {}, "">;
type CurrencyEth = ReturnType<typeof CurrencyEth>;
export default CurrencyEth;
