import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlipVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <FlipVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FlipVertical: Component<IconComponentProps, {}, "">;
type FlipVertical = ReturnType<typeof FlipVertical>;
export default FlipVertical;
