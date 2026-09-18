import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WarningOctagonIcon` instead.
 *
 * @example
 * ```svelte
 * <WarningOctagon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WarningOctagon: Component<IconComponentProps, {}, "">;
type WarningOctagon = ReturnType<typeof WarningOctagon>;
export default WarningOctagon;
