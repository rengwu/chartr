import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AnchorSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <AnchorSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AnchorSimple: Component<IconComponentProps, {}, "">;
type AnchorSimple = ReturnType<typeof AnchorSimple>;
export default AnchorSimple;
