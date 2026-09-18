import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyGbpIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyGbp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyGbp: Component<IconComponentProps, {}, "">;
type CurrencyGbp = ReturnType<typeof CurrencyGbp>;
export default CurrencyGbp;
