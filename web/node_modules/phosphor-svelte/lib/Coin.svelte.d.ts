import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CoinIcon` instead.
 *
 * @example
 * ```svelte
 * <Coin color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Coin: Component<IconComponentProps, {}, "">;
type Coin = ReturnType<typeof Coin>;
export default Coin;
