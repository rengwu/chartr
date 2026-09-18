import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MegaphoneSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <MegaphoneSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MegaphoneSimple: Component<IconComponentProps, {}, "">;
type MegaphoneSimple = ReturnType<typeof MegaphoneSimple>;
export default MegaphoneSimple;
