import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TipiIcon` instead.
 *
 * @example
 * ```svelte
 * <Tipi color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tipi: Component<IconComponentProps, {}, "">;
type Tipi = ReturnType<typeof Tipi>;
export default Tipi;
