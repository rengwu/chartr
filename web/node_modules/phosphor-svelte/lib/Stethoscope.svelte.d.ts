import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StethoscopeIcon` instead.
 *
 * @example
 * ```svelte
 * <Stethoscope color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Stethoscope: Component<IconComponentProps, {}, "">;
type Stethoscope = ReturnType<typeof Stethoscope>;
export default Stethoscope;
