import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MoneyIcon` instead.
 *
 * @example
 * ```svelte
 * <Money color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Money: Component<IconComponentProps, {}, "">;
type Money = ReturnType<typeof Money>;
export default Money;
