import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BankIcon` instead.
 *
 * @example
 * ```svelte
 * <Bank color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bank: Component<IconComponentProps, {}, "">;
type Bank = ReturnType<typeof Bank>;
export default Bank;
