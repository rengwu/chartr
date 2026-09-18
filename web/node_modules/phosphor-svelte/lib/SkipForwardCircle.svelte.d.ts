import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SkipForwardCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <SkipForwardCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SkipForwardCircle: Component<IconComponentProps, {}, "">;
type SkipForwardCircle = ReturnType<typeof SkipForwardCircle>;
export default SkipForwardCircle;
