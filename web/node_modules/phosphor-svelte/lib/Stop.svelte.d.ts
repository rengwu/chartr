import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StopIcon` instead.
 *
 * @example
 * ```svelte
 * <Stop color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Stop: Component<IconComponentProps, {}, "">;
type Stop = ReturnType<typeof Stop>;
export default Stop;
