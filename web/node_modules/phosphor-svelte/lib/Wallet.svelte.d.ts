import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WalletIcon` instead.
 *
 * @example
 * ```svelte
 * <Wallet color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Wallet: Component<IconComponentProps, {}, "">;
type Wallet = ReturnType<typeof Wallet>;
export default Wallet;
