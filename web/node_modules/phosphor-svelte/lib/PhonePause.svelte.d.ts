import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PhonePauseIcon` instead.
 *
 * @example
 * ```svelte
 * <PhonePause color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PhonePause: Component<IconComponentProps, {}, "">;
type PhonePause = ReturnType<typeof PhonePause>;
export default PhonePause;
