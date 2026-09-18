import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LineVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <LineVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LineVertical: Component<IconComponentProps, {}, "">;
type LineVertical = ReturnType<typeof LineVertical>;
export default LineVertical;
