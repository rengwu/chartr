import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `XCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <XCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const XCircle: Component<IconComponentProps, {}, "">;
type XCircle = ReturnType<typeof XCircle>;
export default XCircle;
