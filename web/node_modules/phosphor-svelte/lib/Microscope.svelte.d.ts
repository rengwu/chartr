import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MicroscopeIcon` instead.
 *
 * @example
 * ```svelte
 * <Microscope color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Microscope: Component<IconComponentProps, {}, "">;
type Microscope = ReturnType<typeof Microscope>;
export default Microscope;
