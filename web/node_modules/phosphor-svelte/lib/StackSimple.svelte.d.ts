import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StackSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <StackSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const StackSimple: Component<IconComponentProps, {}, "">;
type StackSimple = ReturnType<typeof StackSimple>;
export default StackSimple;
