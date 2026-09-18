import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PowerIcon` instead.
 *
 * @example
 * ```svelte
 * <Power color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Power: Component<IconComponentProps, {}, "">;
type Power = ReturnType<typeof Power>;
export default Power;
