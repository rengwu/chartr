import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CoinsIcon` instead.
 *
 * @example
 * ```svelte
 * <Coins color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Coins: Component<IconComponentProps, {}, "">;
type Coins = ReturnType<typeof Coins>;
export default Coins;
