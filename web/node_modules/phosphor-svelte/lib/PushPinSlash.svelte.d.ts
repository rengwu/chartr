import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PushPinSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <PushPinSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PushPinSlash: Component<IconComponentProps, {}, "">;
type PushPinSlash = ReturnType<typeof PushPinSlash>;
export default PushPinSlash;
