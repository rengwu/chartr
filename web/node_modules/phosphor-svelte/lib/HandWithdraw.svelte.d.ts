import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandWithdrawIcon` instead.
 *
 * @example
 * ```svelte
 * <HandWithdraw color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandWithdraw: Component<IconComponentProps, {}, "">;
type HandWithdraw = ReturnType<typeof HandWithdraw>;
export default HandWithdraw;
