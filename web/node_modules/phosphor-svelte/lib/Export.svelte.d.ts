import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ExportIcon` instead.
 *
 * @example
 * ```svelte
 * <Export color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Export: Component<IconComponentProps, {}, "">;
type Export = ReturnType<typeof Export>;
export default Export;
