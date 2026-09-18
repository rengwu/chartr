import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SignpostIcon` instead.
 *
 * @example
 * ```svelte
 * <Signpost color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Signpost: Component<IconComponentProps, {}, "">;
type Signpost = ReturnType<typeof Signpost>;
export default Signpost;
