import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BoundingBoxIcon` instead.
 *
 * @example
 * ```svelte
 * <BoundingBox color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BoundingBox: Component<IconComponentProps, {}, "">;
type BoundingBox = ReturnType<typeof BoundingBox>;
export default BoundingBox;
