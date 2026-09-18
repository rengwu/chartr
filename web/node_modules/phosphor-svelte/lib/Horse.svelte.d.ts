import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HorseIcon` instead.
 *
 * @example
 * ```svelte
 * <Horse color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Horse: Component<IconComponentProps, {}, "">;
type Horse = ReturnType<typeof Horse>;
export default Horse;
