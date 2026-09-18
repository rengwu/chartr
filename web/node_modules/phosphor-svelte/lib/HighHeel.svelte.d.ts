import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HighHeelIcon` instead.
 *
 * @example
 * ```svelte
 * <HighHeel color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HighHeel: Component<IconComponentProps, {}, "">;
type HighHeel = ReturnType<typeof HighHeel>;
export default HighHeel;
