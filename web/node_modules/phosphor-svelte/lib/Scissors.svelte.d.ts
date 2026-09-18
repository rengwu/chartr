import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ScissorsIcon` instead.
 *
 * @example
 * ```svelte
 * <Scissors color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Scissors: Component<IconComponentProps, {}, "">;
type Scissors = ReturnType<typeof Scissors>;
export default Scissors;
