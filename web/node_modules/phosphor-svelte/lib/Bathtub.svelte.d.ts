import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BathtubIcon` instead.
 *
 * @example
 * ```svelte
 * <Bathtub color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bathtub: Component<IconComponentProps, {}, "">;
type Bathtub = ReturnType<typeof Bathtub>;
export default Bathtub;
