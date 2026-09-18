import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PushPinSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <PushPinSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PushPinSimple: Component<IconComponentProps, {}, "">;
type PushPinSimple = ReturnType<typeof PushPinSimple>;
export default PushPinSimple;
