import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShuffleSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <ShuffleSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShuffleSimple: Component<IconComponentProps, {}, "">;
type ShuffleSimple = ReturnType<typeof ShuffleSimple>;
export default ShuffleSimple;
