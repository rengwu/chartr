import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CubeIcon` instead.
 *
 * @example
 * ```svelte
 * <Cube color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cube: Component<IconComponentProps, {}, "">;
type Cube = ReturnType<typeof Cube>;
export default Cube;
