import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CylinderIcon` instead.
 *
 * @example
 * ```svelte
 * <Cylinder color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cylinder: Component<IconComponentProps, {}, "">;
type Cylinder = ReturnType<typeof Cylinder>;
export default Cylinder;
