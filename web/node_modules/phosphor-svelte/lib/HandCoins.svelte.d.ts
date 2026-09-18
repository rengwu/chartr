import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandCoinsIcon` instead.
 *
 * @example
 * ```svelte
 * <HandCoins color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandCoins: Component<IconComponentProps, {}, "">;
type HandCoins = ReturnType<typeof HandCoins>;
export default HandCoins;
