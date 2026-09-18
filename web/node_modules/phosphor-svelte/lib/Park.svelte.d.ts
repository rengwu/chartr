import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ParkIcon` instead.
 *
 * @example
 * ```svelte
 * <Park color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Park: Component<IconComponentProps, {}, "">;
type Park = ReturnType<typeof Park>;
export default Park;
