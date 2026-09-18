import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyJpyIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyJpy color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyJpy: Component<IconComponentProps, {}, "">;
type CurrencyJpy = ReturnType<typeof CurrencyJpy>;
export default CurrencyJpy;
