import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MoneyWavyIcon` instead.
 *
 * @example
 * ```svelte
 * <MoneyWavy color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MoneyWavy: Component<IconComponentProps, {}, "">;
type MoneyWavy = ReturnType<typeof MoneyWavy>;
export default MoneyWavy;
