import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BreadIcon` instead.
 *
 * @example
 * ```svelte
 * <Bread color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bread: Component<IconComponentProps, {}, "">;
type Bread = ReturnType<typeof Bread>;
export default Bread;
