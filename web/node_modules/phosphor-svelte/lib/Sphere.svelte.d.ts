import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SphereIcon` instead.
 *
 * @example
 * ```svelte
 * <Sphere color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sphere: Component<IconComponentProps, {}, "">;
type Sphere = ReturnType<typeof Sphere>;
export default Sphere;
