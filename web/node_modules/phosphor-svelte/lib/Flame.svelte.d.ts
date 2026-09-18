import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlameIcon` instead.
 *
 * @example
 * ```svelte
 * <Flame color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Flame: Component<IconComponentProps, {}, "">;
type Flame = ReturnType<typeof Flame>;
export default Flame;
