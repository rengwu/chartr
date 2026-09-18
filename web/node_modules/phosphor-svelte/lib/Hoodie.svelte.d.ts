import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HoodieIcon` instead.
 *
 * @example
 * ```svelte
 * <Hoodie color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Hoodie: Component<IconComponentProps, {}, "">;
type Hoodie = ReturnType<typeof Hoodie>;
export default Hoodie;
