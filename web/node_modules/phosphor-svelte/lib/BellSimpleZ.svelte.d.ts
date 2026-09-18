import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BellSimpleZIcon` instead.
 *
 * @example
 * ```svelte
 * <BellSimpleZ color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BellSimpleZ: Component<IconComponentProps, {}, "">;
type BellSimpleZ = ReturnType<typeof BellSimpleZ>;
export default BellSimpleZ;
