import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PiggyBankIcon` instead.
 *
 * @example
 * ```svelte
 * <PiggyBank color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PiggyBank: Component<IconComponentProps, {}, "">;
type PiggyBank = ReturnType<typeof PiggyBank>;
export default PiggyBank;
