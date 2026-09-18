import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VisorIcon` instead.
 *
 * @example
 * ```svelte
 * <Visor color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Visor: Component<IconComponentProps, {}, "">;
type Visor = ReturnType<typeof Visor>;
export default Visor;
