import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SparkleIcon` instead.
 *
 * @example
 * ```svelte
 * <Sparkle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sparkle: Component<IconComponentProps, {}, "">;
type Sparkle = ReturnType<typeof Sparkle>;
export default Sparkle;
