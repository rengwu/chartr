import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BellSimpleRingingIcon` instead.
 *
 * @example
 * ```svelte
 * <BellSimpleRinging color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BellSimpleRinging: Component<IconComponentProps, {}, "">;
type BellSimpleRinging = ReturnType<typeof BellSimpleRinging>;
export default BellSimpleRinging;
