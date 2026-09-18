import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BellZIcon` instead.
 *
 * @example
 * ```svelte
 * <BellZ color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BellZ: Component<IconComponentProps, {}, "">;
type BellZ = ReturnType<typeof BellZ>;
export default BellZ;
