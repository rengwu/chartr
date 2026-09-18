import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PaintBrushHouseholdIcon` instead.
 *
 * @example
 * ```svelte
 * <PaintBrushHousehold color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PaintBrushHousehold: Component<IconComponentProps, {}, "">;
type PaintBrushHousehold = ReturnType<typeof PaintBrushHousehold>;
export default PaintBrushHousehold;
