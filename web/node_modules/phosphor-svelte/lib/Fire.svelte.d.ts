import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FireIcon` instead.
 *
 * @example
 * ```svelte
 * <Fire color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Fire: Component<IconComponentProps, {}, "">;
type Fire = ReturnType<typeof Fire>;
export default Fire;
