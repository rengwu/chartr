import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LineSegmentIcon` instead.
 *
 * @example
 * ```svelte
 * <LineSegment color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LineSegment: Component<IconComponentProps, {}, "">;
type LineSegment = ReturnType<typeof LineSegment>;
export default LineSegment;
