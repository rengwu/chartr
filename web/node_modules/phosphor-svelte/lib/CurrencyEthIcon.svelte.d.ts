import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <CurrencyEthIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CurrencyEthIcon: Component<IconComponentProps, {}, "">;
type CurrencyEthIcon = ReturnType<typeof CurrencyEthIcon>;
export default CurrencyEthIcon;
