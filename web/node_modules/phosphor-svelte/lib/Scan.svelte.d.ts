import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ScanIcon` instead.
 *
 * @example
 * ```svelte
 * <Scan color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Scan: Component<IconComponentProps, {}, "">;
type Scan = ReturnType<typeof Scan>;
export default Scan;
