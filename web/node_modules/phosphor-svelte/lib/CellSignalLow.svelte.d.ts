import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CellSignalLowIcon` instead.
 *
 * @example
 * ```svelte
 * <CellSignalLow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CellSignalLow: Component<IconComponentProps, {}, "">;
type CellSignalLow = ReturnType<typeof CellSignalLow>;
export default CellSignalLow;
