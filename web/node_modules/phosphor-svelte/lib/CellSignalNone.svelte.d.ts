import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CellSignalNoneIcon` instead.
 *
 * @example
 * ```svelte
 * <CellSignalNone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CellSignalNone: Component<IconComponentProps, {}, "">;
type CellSignalNone = ReturnType<typeof CellSignalNone>;
export default CellSignalNone;
