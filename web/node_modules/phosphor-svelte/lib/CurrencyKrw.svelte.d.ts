import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyKrwIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyKrw color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyKrw: Component<IconComponentProps, {}, "">;
type CurrencyKrw = ReturnType<typeof CurrencyKrw>;
export default CurrencyKrw;
