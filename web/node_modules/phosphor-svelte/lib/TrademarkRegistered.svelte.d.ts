import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrademarkRegisteredIcon` instead.
 *
 * @example
 * ```svelte
 * <TrademarkRegistered color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrademarkRegistered: Component<IconComponentProps, {}, "">;
type TrademarkRegistered = ReturnType<typeof TrademarkRegistered>;
export default TrademarkRegistered;
