import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `IntersectSquareIcon` instead.
 *
 * @example
 * ```svelte
 * <IntersectSquare color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const IntersectSquare: Component<IconComponentProps, {}, "">;
type IntersectSquare = ReturnType<typeof IntersectSquare>;
export default IntersectSquare;
