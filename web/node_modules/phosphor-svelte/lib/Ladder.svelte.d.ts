import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LadderIcon` instead.
 *
 * @example
 * ```svelte
 * <Ladder color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Ladder: Component<IconComponentProps, {}, "">;
type Ladder = ReturnType<typeof Ladder>;
export default Ladder;
