import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyCnyIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyCny color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyCny: Component<IconComponentProps, {}, "">;
type CurrencyCny = ReturnType<typeof CurrencyCny>;
export default CurrencyCny;
