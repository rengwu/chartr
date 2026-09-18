import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EyedropperIcon` instead.
 *
 * @example
 * ```svelte
 * <Eyedropper color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Eyedropper: Component<IconComponentProps, {}, "">;
type Eyedropper = ReturnType<typeof Eyedropper>;
export default Eyedropper;
