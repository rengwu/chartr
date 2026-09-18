import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PaintBrushBroadIcon` instead.
 *
 * @example
 * ```svelte
 * <PaintBrushBroad color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PaintBrushBroad: Component<IconComponentProps, {}, "">;
type PaintBrushBroad = ReturnType<typeof PaintBrushBroad>;
export default PaintBrushBroad;
