import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SkullIcon` instead.
 *
 * @example
 * ```svelte
 * <Skull color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Skull: Component<IconComponentProps, {}, "">;
type Skull = ReturnType<typeof Skull>;
export default Skull;
