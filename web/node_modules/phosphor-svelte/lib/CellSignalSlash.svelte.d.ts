import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CellSignalSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <CellSignalSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CellSignalSlash: Component<IconComponentProps, {}, "">;
type CellSignalSlash = ReturnType<typeof CellSignalSlash>;
export default CellSignalSlash;
