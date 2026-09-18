import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LineSegmentsIcon` instead.
 *
 * @example
 * ```svelte
 * <LineSegments color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LineSegments: Component<IconComponentProps, {}, "">;
type LineSegments = ReturnType<typeof LineSegments>;
export default LineSegments;
