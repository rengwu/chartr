import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BagSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <BagSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BagSimple: Component<IconComponentProps, {}, "">;
type BagSimple = ReturnType<typeof BagSimple>;
export default BagSimple;
