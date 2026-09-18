import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MouseIcon` instead.
 *
 * @example
 * ```svelte
 * <Mouse color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Mouse: Component<IconComponentProps, {}, "">;
type Mouse = ReturnType<typeof Mouse>;
export default Mouse;
