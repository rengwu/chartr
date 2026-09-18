import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MarkerCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <MarkerCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MarkerCircle: Component<IconComponentProps, {}, "">;
type MarkerCircle = ReturnType<typeof MarkerCircle>;
export default MarkerCircle;
