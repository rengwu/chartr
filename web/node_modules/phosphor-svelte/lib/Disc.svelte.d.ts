import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DiscIcon` instead.
 *
 * @example
 * ```svelte
 * <Disc color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Disc: Component<IconComponentProps, {}, "">;
type Disc = ReturnType<typeof Disc>;
export default Disc;
