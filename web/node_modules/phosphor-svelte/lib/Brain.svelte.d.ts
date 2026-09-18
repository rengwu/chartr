import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BrainIcon` instead.
 *
 * @example
 * ```svelte
 * <Brain color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Brain: Component<IconComponentProps, {}, "">;
type Brain = ReturnType<typeof Brain>;
export default Brain;
