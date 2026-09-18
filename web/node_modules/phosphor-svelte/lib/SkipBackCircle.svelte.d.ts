import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SkipBackCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <SkipBackCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SkipBackCircle: Component<IconComponentProps, {}, "">;
type SkipBackCircle = ReturnType<typeof SkipBackCircle>;
export default SkipBackCircle;
