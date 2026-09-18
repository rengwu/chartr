import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DropSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <DropSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DropSimple: Component<IconComponentProps, {}, "">;
type DropSimple = ReturnType<typeof DropSimple>;
export default DropSimple;
