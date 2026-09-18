import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyDollarSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyDollarSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyDollarSimple: Component<IconComponentProps, {}, "">;
type CurrencyDollarSimple = ReturnType<typeof CurrencyDollarSimple>;
export default CurrencyDollarSimple;
