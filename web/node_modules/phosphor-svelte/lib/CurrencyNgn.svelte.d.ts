import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyNgnIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyNgn color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyNgn: Component<IconComponentProps, {}, "">;
type CurrencyNgn = ReturnType<typeof CurrencyNgn>;
export default CurrencyNgn;
