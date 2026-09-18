import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PopcornIcon` instead.
 *
 * @example
 * ```svelte
 * <Popcorn color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Popcorn: Component<IconComponentProps, {}, "">;
type Popcorn = ReturnType<typeof Popcorn>;
export default Popcorn;
