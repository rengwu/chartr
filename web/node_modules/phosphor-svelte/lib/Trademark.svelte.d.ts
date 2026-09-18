import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrademarkIcon` instead.
 *
 * @example
 * ```svelte
 * <Trademark color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Trademark: Component<IconComponentProps, {}, "">;
type Trademark = ReturnType<typeof Trademark>;
export default Trademark;
