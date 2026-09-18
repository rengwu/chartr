import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TramIcon` instead.
 *
 * @example
 * ```svelte
 * <Tram color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tram: Component<IconComponentProps, {}, "">;
type Tram = ReturnType<typeof Tram>;
export default Tram;
