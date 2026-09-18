import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `OctagonIcon` instead.
 *
 * @example
 * ```svelte
 * <Octagon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Octagon: Component<IconComponentProps, {}, "">;
type Octagon = ReturnType<typeof Octagon>;
export default Octagon;
