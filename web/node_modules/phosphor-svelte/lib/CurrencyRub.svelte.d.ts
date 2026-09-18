import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CurrencyRubIcon` instead.
 *
 * @example
 * ```svelte
 * <CurrencyRub color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyRub: Component<IconComponentProps, {}, "">;
type CurrencyRub = ReturnType<typeof CurrencyRub>;
export default CurrencyRub;
