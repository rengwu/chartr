import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShrimpIcon` instead.
 *
 * @example
 * ```svelte
 * <Shrimp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Shrimp: Component<IconComponentProps, {}, "">;
type Shrimp = ReturnType<typeof Shrimp>;
export default Shrimp;
