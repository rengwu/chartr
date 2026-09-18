import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RulerIcon` instead.
 *
 * @example
 * ```svelte
 * <Ruler color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Ruler: Component<IconComponentProps, {}, "">;
type Ruler = ReturnType<typeof Ruler>;
export default Ruler;
