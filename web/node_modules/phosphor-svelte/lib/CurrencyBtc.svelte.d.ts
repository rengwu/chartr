import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyBtcIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyBtc color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyBtc: Component<IconComponentProps, {}, "">;
type CurrencyBtc = ReturnType<typeof CurrencyBtc>;
export default CurrencyBtc;
