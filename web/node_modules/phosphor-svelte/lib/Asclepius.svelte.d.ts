import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AsclepiusIcon` instead.
 *
 * @example
 * ```svelte
 * <Asclepius color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Asclepius: Component<IconComponentProps, {}, "">;
type Asclepius = ReturnType<typeof Asclepius>;
export default Asclepius;
