import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ControlIcon` instead.
 *
 * @example
 * ```svelte
 * <Control color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Control: Component<IconComponentProps, {}, "">;
type Control = ReturnType<typeof Control>;
export default Control;
