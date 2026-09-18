import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PenNibIcon` instead.
 *
 * @example
 * ```svelte
 * <PenNib color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PenNib: Component<IconComponentProps, {}, "">;
type PenNib = ReturnType<typeof PenNib>;
export default PenNib;
