import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlipHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <FlipHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FlipHorizontal: Component<IconComponentProps, {}, "">;
type FlipHorizontal = ReturnType<typeof FlipHorizontal>;
export default FlipHorizontal;
