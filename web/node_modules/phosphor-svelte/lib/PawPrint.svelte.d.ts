import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PawPrintIcon` instead.
 *
 * @example
 * ```svelte
 * <PawPrint color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PawPrint: Component<IconComponentProps, {}, "">;
type PawPrint = ReturnType<typeof PawPrint>;
export default PawPrint;
