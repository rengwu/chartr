import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CarrotIcon` instead.
 *
 * @example
 * ```svelte
 * <Carrot color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Carrot: Component<IconComponentProps, {}, "">;
type Carrot = ReturnType<typeof Carrot>;
export default Carrot;
