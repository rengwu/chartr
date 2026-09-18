import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EqualsIcon` instead.
 *
 * @example
 * ```svelte
 * <Equals color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Equals: Component<IconComponentProps, {}, "">;
type Equals = ReturnType<typeof Equals>;
export default Equals;
