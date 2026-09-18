import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AirplaneIcon` instead.
 *
 * @example
 * ```svelte
 * <Airplane color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Airplane: Component<IconComponentProps, {}, "">;
type Airplane = ReturnType<typeof Airplane>;
export default Airplane;
