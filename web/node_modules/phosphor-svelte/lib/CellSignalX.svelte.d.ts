import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CellSignalXIcon` instead.
 *
 * @example
 * ```svelte
 * <CellSignalX color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CellSignalX: Component<IconComponentProps, {}, "">;
type CellSignalX = ReturnType<typeof CellSignalX>;
export default CellSignalX;
