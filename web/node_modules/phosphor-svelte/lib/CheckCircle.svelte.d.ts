import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CheckCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <CheckCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CheckCircle: Component<IconComponentProps, {}, "">;
type CheckCircle = ReturnType<typeof CheckCircle>;
export default CheckCircle;
