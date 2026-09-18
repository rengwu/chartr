import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RewindCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <RewindCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RewindCircle: Component<IconComponentProps, {}, "">;
type RewindCircle = ReturnType<typeof RewindCircle>;
export default RewindCircle;
