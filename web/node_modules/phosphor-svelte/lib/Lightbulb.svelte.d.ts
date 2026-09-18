import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LightbulbIcon` instead.
 *
 * @example
 * ```svelte
 * <Lightbulb color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Lightbulb: Component<IconComponentProps, {}, "">;
type Lightbulb = ReturnType<typeof Lightbulb>;
export default Lightbulb;
