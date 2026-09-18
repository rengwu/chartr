import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PaintBucketIcon` instead.
 *
 * @example
 * ```svelte
 * <PaintBucket color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PaintBucket: Component<IconComponentProps, {}, "">;
type PaintBucket = ReturnType<typeof PaintBucket>;
export default PaintBucket;
