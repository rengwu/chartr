import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyKztIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyKzt color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyKzt: Component<IconComponentProps, {}, "">;
type CurrencyKzt = ReturnType<typeof CurrencyKzt>;
export default CurrencyKzt;
