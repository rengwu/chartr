import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PushPinSimpleSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <PushPinSimpleSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PushPinSimpleSlash: Component<IconComponentProps, {}, "">;
type PushPinSimpleSlash = ReturnType<typeof PushPinSimpleSlash>;
export default PushPinSimpleSlash;
