import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PerspectiveIcon` instead.
 *
 * @example
 * ```svelte
 * <Perspective color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Perspective: Component<IconComponentProps, {}, "">;
type Perspective = ReturnType<typeof Perspective>;
export default Perspective;
